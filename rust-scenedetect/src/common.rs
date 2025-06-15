//! Common types, utilities, and error handling for the scene detection library.
//! 
//! This module provides the foundational types used throughout the codebase,
//! including frame timecodes, scene cuts, and comprehensive error handling.

use tracing::{instrument, debug};

/// Represents a specific point in time within a video
/// 
/// This type encapsulates both the frame number and framerate information,
/// allowing for precise temporal calculations and conversions.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameTimecode {
    frame_number: u32,
    fps: f64,
}

impl FrameTimecode {
    /// Create a new FrameTimecode
    /// 
    /// # Arguments
    /// * `frame_number` - The frame number (0-indexed)
    /// * `fps` - Frames per second of the video
    /// 
    /// # Panics
    /// Panics if fps is not positive (fail-fast approach)
    #[instrument]
    pub fn new(frame_number: u32, fps: f64) -> Self {
        assert!(fps > 0.0, "FPS must be positive, got: {}", fps);
        debug!("Created FrameTimecode: frame={}, fps={}", frame_number, fps);
        
        Self { frame_number, fps }
    }
    
    /// Get the frame number
    pub fn frame_number(&self) -> u32 {
        self.frame_number
    }
    
    /// Get the framerate
    pub fn fps(&self) -> f64 {
        self.fps
    }
    
    /// Convert to seconds since start of video
    #[instrument(skip(self))]
    pub fn seconds(&self) -> f64 {
        let seconds = self.frame_number as f64 / self.fps;
        debug!("Frame {} at {}fps = {}s", self.frame_number, self.fps, seconds);
        seconds
    }
    
    /// Convert to milliseconds since start of video
    #[instrument(skip(self))]
    pub fn milliseconds(&self) -> f64 {
        self.seconds() * 1000.0
    }
}

/// Represents a detected scene boundary
/// 
/// Each scene cut marks the transition point between two scenes.
/// The end timecode is optional and typically filled when the next cut is found
/// or when the video ends.
#[derive(Debug, Clone)]
pub struct SceneCut {
    pub start: FrameTimecode,
    pub end: Option<FrameTimecode>,
}

impl SceneCut {
    /// Create a new scene cut with just a start timecode
    #[instrument]
    pub fn new(start: FrameTimecode) -> Self {
        debug!("Created SceneCut at frame {}", start.frame_number());
        Self {
            start,
            end: None,
        }
    }
    
    /// Create a complete scene cut with both start and end timecodes
    #[instrument]
    pub fn new_complete(start: FrameTimecode, end: FrameTimecode) -> Self {
        assert_eq!(start.fps(), end.fps(), "Start and end FPS must match");
        assert!(end.frame_number() > start.frame_number(), 
                "End frame must be after start frame");
        
        debug!("Created complete SceneCut: frames {}-{}", 
               start.frame_number(), end.frame_number());
        
        Self {
            start,
            end: Some(end),
        }
    }
    
    /// Get the duration of this scene in frames
    /// Returns None if end timecode is not set
    pub fn duration_frames(&self) -> Option<u32> {
        self.end.as_ref().map(|end| end.frame_number() - self.start.frame_number())
    }
    
    /// Get the duration of this scene in seconds
    /// Returns None if end timecode is not set
    pub fn duration_seconds(&self) -> Option<f64> {
        self.duration_frames().map(|frames| frames as f64 / self.start.fps())
    }
}

/// All possible errors from the scene detection system
/// 
/// This comprehensive error type covers all failure modes in the detection pipeline,
/// from video I/O issues to algorithm failures.
#[derive(Debug, thiserror::Error)]
pub enum SceneDetectError {
    #[error("Failed to open video file: {path}")]
    VideoOpenFailed { path: String },
    
    #[error("Video file not found: {path}")]
    VideoNotFound { path: String },
    
    #[error("Invalid video format or corrupted file: {path}")]
    InvalidVideoFormat { path: String },
    
    #[error("OpenCV error: {0}")]
    OpenCvError(#[from] opencv::Error),
    
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },
    
    #[error("Frame processing failed at frame {frame}: {reason}")]
    FrameProcessingFailed { frame: u32, reason: String },
    
    #[error("No frames found in video")]
    EmptyVideo,
    
    #[error("Unsupported video codec or format")]
    UnsupportedFormat,
    
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl SceneDetectError {
    /// Create a configuration error with a descriptive message
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::InvalidConfig { message: message.into() }
    }
    
    /// Create a frame processing error
    pub fn frame_error(frame: u32, reason: impl Into<String>) -> Self {
        Self::FrameProcessingFailed { 
            frame, 
            reason: reason.into() 
        }
    }
    
    /// Create an internal error (for unexpected conditions)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError { message: message.into() }
    }
}

/// Convenient Result type for scene detection operations
pub type Result<T> = std::result::Result<T, SceneDetectError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_timecode_creation() {
        let tc = FrameTimecode::new(100, 25.0);
        assert_eq!(tc.frame_number(), 100);
        assert_eq!(tc.fps(), 25.0);
        assert_eq!(tc.seconds(), 4.0);
        assert_eq!(tc.milliseconds(), 4000.0);
    }
    
    #[test]
    #[should_panic(expected = "FPS must be positive")]
    fn test_frame_timecode_invalid_fps_zero() {
        FrameTimecode::new(100, 0.0);
    }
    
    #[test] 
    #[should_panic(expected = "FPS must be positive")]
    fn test_frame_timecode_invalid_fps_negative() {
        FrameTimecode::new(100, -1.0);
    }
    
    #[test]
    fn test_frame_timecode_edge_cases() {
        // Test frame 0
        let tc = FrameTimecode::new(0, 30.0);
        assert_eq!(tc.seconds(), 0.0);
        
        // Test high frame number
        let tc = FrameTimecode::new(1000000, 60.0);
        assert_eq!(tc.frame_number(), 1000000);
        
        // Test fractional FPS
        let tc = FrameTimecode::new(100, 29.97);
        assert!((tc.seconds() - 3.336_669_999_999_999_8).abs() < 1e-10);
    }
    
    #[test]
    fn test_scene_cut_creation() {
        let start = FrameTimecode::new(100, 25.0);
        let cut = SceneCut::new(start.clone());
        
        assert_eq!(cut.start.frame_number(), 100);
        assert!(cut.end.is_none());
        assert!(cut.duration_frames().is_none());
        assert!(cut.duration_seconds().is_none());
    }
    
    #[test]
    fn test_scene_cut_complete() {
        let start = FrameTimecode::new(100, 25.0);
        let end = FrameTimecode::new(200, 25.0);
        let cut = SceneCut::new_complete(start, end);
        
        assert_eq!(cut.start.frame_number(), 100);
        assert_eq!(cut.end.as_ref().unwrap().frame_number(), 200);
        assert_eq!(cut.duration_frames(), Some(100));
        assert_eq!(cut.duration_seconds(), Some(4.0));
    }
    
    #[test]
    #[should_panic(expected = "Start and end FPS must match")]
    fn test_scene_cut_mismatched_fps() {
        let start = FrameTimecode::new(100, 25.0);
        let end = FrameTimecode::new(200, 30.0);
        SceneCut::new_complete(start, end);
    }
    
    #[test]
    #[should_panic(expected = "End frame must be after start frame")]
    fn test_scene_cut_invalid_order() {
        let start = FrameTimecode::new(200, 25.0);
        let end = FrameTimecode::new(100, 25.0);
        SceneCut::new_complete(start, end);
    }
    
    #[test]
    fn test_error_types() {
        let error = SceneDetectError::config_error("Invalid threshold");
        assert!(matches!(error, SceneDetectError::InvalidConfig { .. }));
        
        let error = SceneDetectError::frame_error(42, "Processing failed");
        assert!(matches!(error, SceneDetectError::FrameProcessingFailed { frame: 42, .. }));
        
        let error = SceneDetectError::internal_error("Unexpected condition");
        assert!(matches!(error, SceneDetectError::InternalError { .. }));
    }
    
    #[test]
    fn test_error_display() {
        let error = SceneDetectError::VideoNotFound { 
            path: "test.mp4".to_string() 
        };
        assert_eq!(error.to_string(), "Video file not found: test.mp4");
        
        let error = SceneDetectError::config_error("Threshold must be positive");
        assert_eq!(error.to_string(), "Invalid configuration: Threshold must be positive");
    }
}