//! Rust implementation of PySceneDetect's ContentDetector
//! 
//! This crate provides a minimal but performant scene detection library
//! that matches the interface and behavior of PySceneDetect's ContentDetector.
//! 
//! # Quick Start
//! 
//! ```rust,no_run
//! use rust_scenedetect::{detect_scene_changes, ContentDetector, detect};
//! 
//! // Simple usage matching Python pattern
//! let frame_numbers = detect_scene_changes("video.mp4")?;
//! println!("Scene changes at frames: {:?}", frame_numbers);
//! 
//! // Advanced usage with custom detector
//! let detector = ContentDetector::new(30.0); // Custom threshold
//! let scene_list = detect("video.mp4", detector)?;
//! for scene in scene_list {
//!     println!("Scene starts at frame {}", scene.start.frame_number());
//! }
//! # Ok::<(), rust_scenedetect::SceneDetectError>(())
//! ```

mod common;
mod video_stream;
mod content_detector;
mod flash_filter;

// Re-export main types for public API
pub use common::{FrameTimecode, SceneCut, SceneDetectError, Result};
pub use content_detector::{ContentDetector, ComponentWeights};
pub use flash_filter::{FlashFilter, FilterMode};
pub use video_stream::VideoStream;

use tracing::{instrument, info, debug, warn};

/// Main detection function matching PySceneDetect's `detect()` interface
/// 
/// This function provides the core scene detection functionality, processing
/// an entire video file and returning a list of detected scene boundaries.
/// 
/// # Arguments
/// * `video_path` - Path to the video file to analyze
/// * `detector` - ContentDetector instance with desired settings
/// 
/// # Returns
/// * `Result<Vec<SceneCut>>` - List of detected scene cuts with timing information
/// 
/// # Errors
/// * `VideoNotFound` - If the video file doesn't exist
/// * `VideoOpenFailed` - If OpenCV can't open the video
/// * `FrameProcessingFailed` - If frame analysis fails
/// 
/// # Example
/// ```rust,no_run
/// use rust_scenedetect::{ContentDetector, detect};
/// 
/// let detector = ContentDetector::new(27.0);
/// let scenes = detect("my_video.mp4", detector)?;
/// 
/// for (i, scene) in scenes.iter().enumerate() {
///     println!("Scene {}: starts at frame {}", i + 1, scene.start.frame_number());
/// }
/// # Ok::<(), rust_scenedetect::SceneDetectError>(())
/// ```
#[instrument(skip(detector))]
pub fn detect(video_path: &str, mut detector: ContentDetector) -> Result<Vec<SceneCut>> {
    info!("Starting scene detection for: {}", video_path);
    
    // Reset detector state in case it was used before
    detector.reset();
    
    let mut video_stream = VideoStream::open(video_path)?;
    let mut cuts = Vec::new();
    
    info!("Video properties: {}x{} at {:.2}fps, {} frames total",
          video_stream.width(), video_stream.height(), 
          video_stream.fps(), video_stream.frame_count());
    
    // Process all frames
    let mut frames_processed = 0;
    let total_frames = video_stream.frame_count();
    
    while let Some(frame) = video_stream.read_frame()? {
        let timecode = FrameTimecode::new(
            video_stream.current_frame() as u32, 
            video_stream.fps()
        );
        
        if let Some(cut_timecode) = detector.process_frame(&frame, timecode)? {
            debug!("Scene cut detected at frame {} ({:.2}s)", 
                   cut_timecode.frame_number(), cut_timecode.seconds());
            
            cuts.push(SceneCut::new(cut_timecode));
        }
        
        frames_processed += 1;
        
        // Log progress for long videos
        if frames_processed % 1000 == 0 {
            let progress = video_stream.progress_percent();
            debug!("Processed {}/{} frames ({:.1}%)", 
                   frames_processed, total_frames, progress);
        }
    }
    
    // Complete scene information by setting end times
    complete_scene_cuts(&mut cuts, video_stream.fps(), video_stream.frame_count());
    
    info!("Scene detection completed. Found {} cuts in {} frames", 
          cuts.len(), frames_processed);
    
    Ok(cuts)
}

/// Helper function matching your current Python usage pattern
/// 
/// This function provides a simple interface that exactly matches your existing
/// `detect_scene_changes()` function, returning just the frame numbers of scene cuts.
/// 
/// # Arguments
/// * `video_path` - Path to the video file to analyze
/// 
/// # Returns
/// * `Result<Vec<u32>>` - Frame numbers where scene cuts were detected
/// 
/// # Example
/// ```rust,no_run
/// use rust_scenedetect::detect_scene_changes;
/// 
/// // Drop-in replacement for your Python function
/// let frame_numbers = detect_scene_changes("video.mp4")?;
/// println!("Scene changes at frames: {:?}", frame_numbers);
/// # Ok::<(), rust_scenedetect::SceneDetectError>(())
/// ```
#[instrument]
pub fn detect_scene_changes(video_path: &str) -> Result<Vec<u32>> {
    let detector = ContentDetector::new(27.0); // PySceneDetect default threshold
    let scene_list = detect(video_path, detector)?;
    
    let frame_numbers: Vec<u32> = scene_list
        .iter()
        .map(|scene| scene.start.frame_number())
        .collect();
    
    info!("Extracted {} scene change frame numbers", frame_numbers.len());
    
    Ok(frame_numbers)
}

/// Get video information without performing scene detection
/// 
/// This function provides basic video metadata that might be useful
/// for configuring detection parameters or validating input.
/// 
/// # Arguments
/// * `video_path` - Path to the video file to analyze
/// 
/// # Returns
/// * `Result<VideoInfo>` - Video metadata including FPS, frame count, and dimensions
#[instrument]
pub fn get_video_info(video_path: &str) -> Result<VideoInfo> {
    let video_stream = VideoStream::open(video_path)?;
    
    Ok(VideoInfo {
        path: video_path.to_string(),
        fps: video_stream.fps(),
        frame_count: video_stream.frame_count() as u32,
        width: video_stream.width() as u32,
        height: video_stream.height() as u32,
        duration_seconds: video_stream.duration_seconds(),
    })
}

/// Video metadata information
#[derive(Debug, Clone, PartialEq)]
pub struct VideoInfo {
    pub path: String,
    pub fps: f64,
    pub frame_count: u32,
    pub width: u32,
    pub height: u32,
    pub duration_seconds: f64,
}

impl VideoInfo {
    /// Get a human-readable description of the video
    pub fn description(&self) -> String {
        format!(
            "{}x{} at {:.2}fps, {} frames ({:.1}s duration)",
            self.width, self.height, self.fps, self.frame_count, self.duration_seconds
        )
    }
    
    /// Check if this appears to be a valid video configuration
    pub fn is_valid(&self) -> bool {
        self.fps > 0.0 
            && self.frame_count > 0 
            && self.width > 0 
            && self.height > 0 
            && self.duration_seconds > 0.0
    }
}

/// Complete scene cut information by setting end times
/// 
/// This helper function fills in the end times for all scene cuts based on
/// when the next cut occurs (or the video ends).
fn complete_scene_cuts(cuts: &mut [SceneCut], fps: f64, total_frames: i32) {
    if cuts.is_empty() {
        return;
    }
    
    // Set end times for all cuts except the last
    for i in 0..cuts.len() - 1 {
        let next_start_frame = cuts[i + 1].start.frame_number();
        cuts[i].end = Some(FrameTimecode::new(next_start_frame, fps));
    }
    
    // Set end time for the last cut to the end of the video
    if let Some(last_cut) = cuts.last_mut() {
        last_cut.end = Some(FrameTimecode::new(total_frames as u32, fps));
    }
}

/// Initialize tracing for the library
/// 
/// This function sets up logging/tracing for the scene detection library.
/// Call this once at the start of your application to enable debug output.
/// 
/// # Arguments
/// * `level` - Tracing level filter (e.g., "debug", "info", "warn", "error")
pub fn init_tracing(level: &str) {
    use tracing_subscriber::{EnvFilter, fmt};
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    info!("Rust scene detection library initialized with tracing level: {}", level);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_video_info_creation() {
        let info = VideoInfo {
            path: "test.mp4".to_string(),
            fps: 25.0,
            frame_count: 1000,
            width: 1920,
            height: 1080,
            duration_seconds: 40.0,
        };
        
        assert!(info.is_valid());
        assert!(info.description().contains("1920x1080"));
        assert!(info.description().contains("25.00fps"));
        assert!(info.description().contains("1000 frames"));
    }
    
    #[test]
    fn test_video_info_validation() {
        let valid_info = VideoInfo {
            path: "test.mp4".to_string(),
            fps: 30.0,
            frame_count: 100,
            width: 640,
            height: 480,
            duration_seconds: 3.33,
        };
        assert!(valid_info.is_valid());
        
        // Test various invalid configurations
        let invalid_fps = VideoInfo { fps: 0.0, ..valid_info.clone() };
        assert!(!invalid_fps.is_valid());
        
        let invalid_frames = VideoInfo { frame_count: 0, ..valid_info.clone() };
        assert!(!invalid_frames.is_valid());
        
        let invalid_width = VideoInfo { width: 0, ..valid_info.clone() };
        assert!(!invalid_width.is_valid());
        
        let invalid_height = VideoInfo { height: 0, ..valid_info.clone() };
        assert!(!invalid_height.is_valid());
        
        let invalid_duration = VideoInfo { duration_seconds: 0.0, ..valid_info };
        assert!(!invalid_duration.is_valid());
    }
    
    #[test]
    fn test_complete_scene_cuts() {
        let fps = 25.0;
        let total_frames = 1000;
        
        let mut cuts = vec![
            SceneCut::new(FrameTimecode::new(0, fps)),
            SceneCut::new(FrameTimecode::new(250, fps)),
            SceneCut::new(FrameTimecode::new(500, fps)),
        ];
        
        complete_scene_cuts(&mut cuts, fps, total_frames);
        
        // Check that end times were set correctly
        assert_eq!(cuts[0].end.as_ref().unwrap().frame_number(), 250);
        assert_eq!(cuts[1].end.as_ref().unwrap().frame_number(), 500);
        assert_eq!(cuts[2].end.as_ref().unwrap().frame_number(), 1000);
    }
    
    #[test]
    fn test_complete_scene_cuts_empty() {
        let mut cuts: Vec<SceneCut> = vec![];
        complete_scene_cuts(&mut cuts, 25.0, 1000);
        assert!(cuts.is_empty()); // Should not panic
    }
    
    #[test]
    fn test_complete_scene_cuts_single() {
        let fps = 30.0;
        let total_frames = 600;
        
        let mut cuts = vec![
            SceneCut::new(FrameTimecode::new(100, fps)),
        ];
        
        complete_scene_cuts(&mut cuts, fps, total_frames);
        
        assert_eq!(cuts[0].end.as_ref().unwrap().frame_number(), 600);
    }
    
    // Note: Integration tests with actual video files would be in tests/ directory
    // These tests verify the API structure and basic functionality
    
    #[test] 
    fn test_api_functions_exist() {
        // Verify that the main API functions are accessible
        // (This would require actual video files to test fully)
        
        // Test that we can create a ContentDetector
        let detector = ContentDetector::new(27.0);
        assert_eq!(detector.threshold(), 27.0);
        
        // Test that VideoInfo implements expected traits
        let info = VideoInfo {
            path: "test.mp4".to_string(),
            fps: 25.0,
            frame_count: 100,
            width: 640,
            height: 480,
            duration_seconds: 4.0,
        };
        
        // Should be cloneable and debuggable
        let _info_clone = info.clone();
        let _debug_str = format!("{:?}", info);
    }
}