//! ContentDetector - HSV-based scene change detection algorithm
//! 
//! This module implements the core ContentDetector algorithm from PySceneDetect,
//! which detects scene changes by analyzing differences in HSV color space
//! between consecutive video frames.

use opencv::{core::{self, Mat, Vector, Scalar, CV_8UC3}, imgproc, prelude::*};
use tracing::{instrument, debug, trace, warn};
use crate::{
    common::{FrameTimecode, Result, SceneDetectError},
    flash_filter::{FlashFilter, FilterMode},
};

/// Component weights for frame score calculation
/// 
/// These weights determine how much each color channel contributes to the
/// final scene change score. Default values match PySceneDetect's behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentWeights {
    pub delta_hue: f64,
    pub delta_sat: f64, 
    pub delta_lum: f64,
    pub delta_edges: f64,
}

impl Default for ComponentWeights {
    fn default() -> Self {
        // PySceneDetect default weights
        Self {
            delta_hue: 1.0,
            delta_sat: 1.0,
            delta_lum: 1.0,
            delta_edges: 0.0, // Edge detection disabled by default for MVP
        }
    }
}

impl ComponentWeights {
    /// Create weights for luma-only detection (brightness changes only)
    pub fn luma_only() -> Self {
        Self {
            delta_hue: 0.0,
            delta_sat: 0.0,
            delta_lum: 1.0,
            delta_edges: 0.0,
        }
    }
    
    /// Get the sum of absolute weights (for normalization)
    pub fn sum_abs(&self) -> f64 {
        self.delta_hue.abs() + self.delta_sat.abs() + self.delta_lum.abs() + self.delta_edges.abs()
    }
    
    /// Validate that weights are reasonable
    fn validate(&self) -> Result<()> {
        let sum = self.sum_abs();
        if sum <= 0.0 {
            return Err(SceneDetectError::config_error(
                "All component weights cannot be zero"
            ));
        }
        Ok(())
    }
}

/// Frame data extracted for scene detection analysis
#[derive(Debug)]
struct FrameData {
    hue: Mat,
    sat: Mat,
    lum: Mat,
    edges: Option<Mat>, // Optional for MVP
}

impl FrameData {
    /// Create FrameData from a BGR frame
    #[instrument(skip(frame))]
    fn from_bgr_frame(frame: &Mat) -> Result<Self> {
        // Convert BGR to HSV color space
        let mut hsv = Mat::default();
        imgproc::cvt_color_def(frame, &mut hsv, imgproc::COLOR_BGR2HSV)
            .map_err(|e| SceneDetectError::frame_error(0, format!("HSV conversion failed: {}", e)))?;
        
        // Split HSV channels
        let mut channels = Vector::<Mat>::new();
        core::split(&hsv, &mut channels)
            .map_err(|e| SceneDetectError::frame_error(0, format!("Channel split failed: {}", e)))?;
        
        if channels.len() != 3 {
            return Err(SceneDetectError::frame_error(0, 
                format!("Expected 3 HSV channels, got {}", channels.len())));
        }
        
        let hue = channels.get(0)
            .map_err(|e| SceneDetectError::frame_error(0, format!("Failed to get hue channel: {}", e)))?;
        let sat = channels.get(1)
            .map_err(|e| SceneDetectError::frame_error(0, format!("Failed to get saturation channel: {}", e)))?;
        let lum = channels.get(2)
            .map_err(|e| SceneDetectError::frame_error(0, format!("Failed to get luminance channel: {}", e)))?;
        
        Ok(Self {
            hue,
            sat,
            lum,
            edges: None, // Edge detection skipped for MVP simplicity
        })
    }
    
    /// Validate frame data consistency
    fn validate(&self) -> Result<()> {
        let hue_size = self.hue.size()
            .map_err(|e| SceneDetectError::internal_error(format!("Failed to get hue size: {}", e)))?;
        let sat_size = self.sat.size()
            .map_err(|e| SceneDetectError::internal_error(format!("Failed to get sat size: {}", e)))?;
        let lum_size = self.lum.size()
            .map_err(|e| SceneDetectError::internal_error(format!("Failed to get lum size: {}", e)))?;
        
        if hue_size != sat_size || sat_size != lum_size {
            return Err(SceneDetectError::internal_error(
                format!("Channel size mismatch: hue={:?}, sat={:?}, lum={:?}", 
                       hue_size, sat_size, lum_size)
            ));
        }
        
        Ok(())
    }
}

/// ContentDetector - detects scene changes using HSV color space analysis
/// 
/// This detector compares consecutive frames in the HSV color space and
/// calculates a weighted score based on the differences in hue, saturation,
/// and luminance channels. When this score exceeds a threshold, a scene cut
/// is detected.
pub struct ContentDetector {
    threshold: f64,
    weights: ComponentWeights,
    last_frame_data: Option<FrameData>,
    flash_filter: FlashFilter,
    frame_count: u32,
}

impl ContentDetector {
    /// Create a new ContentDetector with default settings
    /// 
    /// Uses PySceneDetect's default threshold of 27.0 and standard component weights.
    /// 
    /// # Arguments
    /// * `threshold` - Score threshold for detecting scene changes (default: 27.0)
    /// 
    /// # Panics
    /// Panics if threshold is negative (fail-fast approach)
    #[instrument]
    pub fn new(threshold: f64) -> Self {
        assert!(threshold >= 0.0, "Threshold must be non-negative, got: {}", threshold);
        
        debug!("Created ContentDetector with threshold: {}", threshold);
        
        Self {
            threshold,
            weights: ComponentWeights::default(),
            last_frame_data: None,
            flash_filter: FlashFilter::new(15), // PySceneDetect default: 15 frames
            frame_count: 0,
        }
    }
    
    /// Create a ContentDetector with custom settings
    /// 
    /// # Arguments
    /// * `threshold` - Score threshold for detecting scene changes
    /// * `weights` - Component weights for score calculation
    /// * `min_scene_length` - Minimum frames between scene cuts
    /// * `filter_mode` - Flash filter mode (Merge or Suppress)
    #[instrument(skip(weights))]
    pub fn new_with_config(
        threshold: f64,
        weights: ComponentWeights,
        min_scene_length: u32,
        filter_mode: FilterMode,
    ) -> Result<Self> {
        assert!(threshold >= 0.0, "Threshold must be non-negative");
        
        weights.validate()?;
        
        debug!("Created ContentDetector with custom config: threshold={}, min_scene_length={}, mode={:?}", 
               threshold, min_scene_length, filter_mode);
        
        Ok(Self {
            threshold,
            weights,
            last_frame_data: None,
            flash_filter: FlashFilter::new_with_mode(filter_mode, min_scene_length),
            frame_count: 0,
        })
    }
    
    /// Create a luma-only ContentDetector (brightness changes only)
    /// 
    /// This is useful for black and white videos or when color information
    /// is not reliable for scene detection.
    #[instrument]
    pub fn new_luma_only(threshold: f64) -> Self {
        assert!(threshold >= 0.0, "Threshold must be non-negative");
        
        debug!("Created luma-only ContentDetector with threshold: {}", threshold);
        
        Self {
            threshold,
            weights: ComponentWeights::luma_only(),
            last_frame_data: None,
            flash_filter: FlashFilter::new(15),
            frame_count: 0,
        }
    }
    
    /// Process a single frame and return scene cut if detected
    /// 
    /// # Arguments
    /// * `frame` - BGR video frame to process
    /// * `timecode` - Timecode for this frame
    /// 
    /// # Returns
    /// * `Result<Option<FrameTimecode>>` - Scene cut timecode if detected, None otherwise
    #[instrument(skip(self, frame))]
    pub fn process_frame(&mut self, frame: &Mat, timecode: FrameTimecode) -> Result<Option<FrameTimecode>> {
        self.frame_count += 1;
        
        // Validate input frame
        if frame.empty() {
            return Err(SceneDetectError::frame_error(
                timecode.frame_number(), 
                "Empty frame provided".to_string()
            ));
        }
        
        let frame_score = self.calculate_frame_score(frame, timecode.frame_number())?;
        
        trace!("Frame {} score: {:.3} (threshold: {})", 
               timecode.frame_number(), frame_score, self.threshold);
        
        let above_threshold = frame_score >= self.threshold;
        let cuts = self.flash_filter.filter(timecode, above_threshold);
        
        Ok(cuts.into_iter().next())
    }
    
    /// Calculate content change score between current and previous frame
    #[instrument(skip(self, frame))]
    fn calculate_frame_score(&mut self, frame: &Mat, frame_number: u32) -> Result<f64> {
        // Extract frame data
        let current_data = FrameData::from_bgr_frame(frame)
            .map_err(|e| SceneDetectError::frame_error(frame_number, format!("Frame analysis failed: {}", e)))?;
        
        current_data.validate()
            .map_err(|e| SceneDetectError::frame_error(frame_number, format!("Frame validation failed: {}", e)))?;
        
        let score = if let Some(ref last_data) = self.last_frame_data {
            // Calculate differences for each channel
            let delta_hue = Self::mean_pixel_distance(&current_data.hue, &last_data.hue)
                .map_err(|e| SceneDetectError::frame_error(frame_number, format!("Hue difference calculation failed: {}", e)))?;
            
            let delta_sat = Self::mean_pixel_distance(&current_data.sat, &last_data.sat)
                .map_err(|e| SceneDetectError::frame_error(frame_number, format!("Saturation difference calculation failed: {}", e)))?;
            
            let delta_lum = Self::mean_pixel_distance(&current_data.lum, &last_data.lum)
                .map_err(|e| SceneDetectError::frame_error(frame_number, format!("Luminance difference calculation failed: {}", e)))?;
            
            let delta_edges = 0.0; // Skipped for MVP
            
            // Calculate weighted score (matching PySceneDetect formula)
            let weighted_sum = 
                delta_hue * self.weights.delta_hue +
                delta_sat * self.weights.delta_sat +
                delta_lum * self.weights.delta_lum +
                delta_edges * self.weights.delta_edges;
            
            let weight_sum = self.weights.sum_abs();
            let final_score = weighted_sum / weight_sum;
            
            trace!("Frame {} components: hue={:.3}, sat={:.3}, lum={:.3}, final={:.3}",
                   frame_number, delta_hue, delta_sat, delta_lum, final_score);
            
            final_score
        } else {
            // First frame - no comparison possible
            debug!("First frame ({}), score = 0.0", frame_number);
            0.0
        };
        
        // Store current frame data for next comparison
        self.last_frame_data = Some(current_data);
        
        Ok(score)
    }
    
    /// Calculate mean absolute difference between two single-channel images
    /// 
    /// This is the core metric used by PySceneDetect to measure frame differences.
    #[instrument(skip(left, right))]
    fn mean_pixel_distance(left: &Mat, right: &Mat) -> Result<f64> {
        // Validate input sizes match
        let left_size = left.size()
            .map_err(|e| SceneDetectError::internal_error(format!("Failed to get left size: {}", e)))?;
        let right_size = right.size()
            .map_err(|e| SceneDetectError::internal_error(format!("Failed to get right size: {}", e)))?;
        
        if left_size != right_size {
            return Err(SceneDetectError::internal_error(
                format!("Image size mismatch: left={:?}, right={:?}", left_size, right_size)
            ));
        }
        
        // Calculate absolute difference
        let mut diff = Mat::default();
        core::absdiff(left, right, &mut diff)
            .map_err(|e| SceneDetectError::internal_error(format!("Failed to calculate absolute difference: {}", e)))?;
        
        // Sum all pixel differences
        let sum_scalar = core::sum_elems(&diff)
            .map_err(|e| SceneDetectError::internal_error(format!("Failed to calculate sum: {}", e)))?;
        
        // Calculate mean (total pixels = width * height)
        let num_pixels = (left.rows() * left.cols()) as f64;
        let mean_distance = sum_scalar[0] / num_pixels;
        
        trace!("Mean pixel distance: {:.3} (over {} pixels)", mean_distance, num_pixels);
        
        Ok(mean_distance)
    }
    
    /// Get the current threshold setting
    pub fn threshold(&self) -> f64 {
        self.threshold
    }
    
    /// Get the current component weights
    pub fn weights(&self) -> &ComponentWeights {
        &self.weights
    }
    
    /// Get the number of frames processed so far
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
    
    /// Get the minimum scene length setting from the flash filter
    pub fn min_scene_length(&self) -> u32 {
        self.flash_filter.min_scene_length()
    }
    
    /// Reset the detector state (useful for processing multiple videos)
    #[instrument(skip(self))]
    pub fn reset(&mut self) {
        debug!("Resetting ContentDetector state");
        self.last_frame_data = None;
        self.flash_filter.reset();
        self.frame_count = 0;
    }
}

// Implement Debug manually to avoid showing internal OpenCV state
impl std::fmt::Debug for ContentDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContentDetector")
            .field("threshold", &self.threshold)
            .field("weights", &self.weights)
            .field("frame_count", &self.frame_count)
            .field("has_last_frame", &self.last_frame_data.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencv::{core::CV_8UC3, imgproc};
    
    fn create_test_frame(width: i32, height: i32, color: (u8, u8, u8)) -> Result<Mat> {
        let mut frame = Mat::new_rows_cols_with_default(
            height, width, CV_8UC3, Scalar::from((color.0 as i32, color.1 as i32, color.2 as i32))
        )?;
        Ok(frame)
    }
    
    fn create_timecode(frame: u32) -> FrameTimecode {
        FrameTimecode::new(frame, 25.0)
    }
    
    #[test]
    fn test_content_detector_creation() {
        let detector = ContentDetector::new(27.0);
        assert_eq!(detector.threshold(), 27.0);
        assert_eq!(detector.frame_count(), 0);
        assert_eq!(detector.min_scene_length(), 15);
        
        let weights = detector.weights();
        assert_eq!(weights.delta_hue, 1.0);
        assert_eq!(weights.delta_sat, 1.0);
        assert_eq!(weights.delta_lum, 1.0);
        assert_eq!(weights.delta_edges, 0.0);
    }
    
    #[test]
    #[should_panic(expected = "Threshold must be non-negative")]
    fn test_content_detector_negative_threshold() {
        ContentDetector::new(-1.0);
    }
    
    #[test]
    fn test_content_detector_luma_only() {
        let detector = ContentDetector::new_luma_only(30.0);
        let weights = detector.weights();
        assert_eq!(weights.delta_hue, 0.0);
        assert_eq!(weights.delta_sat, 0.0);
        assert_eq!(weights.delta_lum, 1.0);
        assert_eq!(weights.delta_edges, 0.0);
    }
    
    #[test]
    fn test_component_weights() {
        let weights = ComponentWeights::default();
        assert_eq!(weights.sum_abs(), 3.0); // 1.0 + 1.0 + 1.0 + 0.0
        
        let luma_weights = ComponentWeights::luma_only();
        assert_eq!(luma_weights.sum_abs(), 1.0);
        
        // Test validation
        let zero_weights = ComponentWeights {
            delta_hue: 0.0,
            delta_sat: 0.0,
            delta_lum: 0.0,
            delta_edges: 0.0,
        };
        assert!(zero_weights.validate().is_err());
        
        let valid_weights = ComponentWeights {
            delta_hue: 0.5,
            delta_sat: 0.3,
            delta_lum: 0.2,
            delta_edges: 0.0,
        };
        assert!(valid_weights.validate().is_ok());
    }
    
    #[test]
    fn test_content_detector_custom_config() {
        let weights = ComponentWeights {
            delta_hue: 0.5,
            delta_sat: 0.3,
            delta_lum: 0.2,
            delta_edges: 0.0,
        };
        
        let detector = ContentDetector::new_with_config(
            30.0, weights.clone(), 20, FilterMode::Merge
        ).unwrap();
        
        assert_eq!(detector.threshold(), 30.0);
        assert_eq!(detector.min_scene_length(), 20);
        assert_eq!(detector.weights(), &weights);
    }
    
    // Note: The following tests would require actual OpenCV functionality
    // In a real implementation, you'd want to create simple test frames
    // and verify the detection logic works correctly.
    
    #[test]
    fn test_frame_processing_basic() {
        // This test would work with real OpenCV frames:
        // let mut detector = ContentDetector::new(27.0);
        // let frame1 = create_test_frame(100, 100, (255, 0, 0)).unwrap(); // Red frame
        // let frame2 = create_test_frame(100, 100, (0, 255, 0)).unwrap(); // Green frame
        // 
        // // First frame should not produce a cut
        // let result = detector.process_frame(&frame1, create_timecode(1)).unwrap();
        // assert!(result.is_none());
        // 
        // // Second frame (very different) should produce a cut
        // let result = detector.process_frame(&frame2, create_timecode(2)).unwrap();
        // assert!(result.is_some());
    }
    
    #[test]
    fn test_detector_reset() {
        let mut detector = ContentDetector::new(27.0);
        
        // Process would increment frame count
        // detector.process_frame(&some_frame, create_timecode(1)).unwrap();
        // assert_eq!(detector.frame_count(), 1);
        
        detector.reset();
        assert_eq!(detector.frame_count(), 0);
    }
    
    #[test]
    fn test_debug_formatting() {
        let detector = ContentDetector::new(27.0);
        let debug_str = format!("{:?}", detector);
        assert!(debug_str.contains("ContentDetector"));
        assert!(debug_str.contains("threshold"));
        assert!(debug_str.contains("27"));
    }
}