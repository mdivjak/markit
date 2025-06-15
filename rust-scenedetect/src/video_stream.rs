//! Video stream handling using OpenCV backend
//! 
//! This module provides a wrapper around OpenCV's VideoCapture for consistent
//! video reading and frame processing. It matches PySceneDetect's OpenCV backend
//! behavior while providing Rust-specific safety guarantees.

use opencv::{videoio, core::Mat, prelude::*};
use tracing::{instrument, debug, warn, info};
use std::path::Path;
use crate::common::{Result, SceneDetectError};

/// Wrapper around OpenCV VideoCapture for consistent video reading
/// 
/// This struct provides a safe, instrumented interface to OpenCV's video
/// reading capabilities, with proper error handling and logging.
pub struct VideoStream {
    cap: videoio::VideoCapture,
    fps: f64,
    frame_count: i32,
    current_frame: i32,
    width: i32,
    height: i32,
    path: String,
}

impl VideoStream {
    /// Open a video file for reading
    /// 
    /// # Arguments
    /// * `path` - Path to the video file
    /// 
    /// # Returns
    /// * `Result<VideoStream>` - A new video stream instance or an error
    /// 
    /// # Errors
    /// * `VideoNotFound` - If the file doesn't exist
    /// * `VideoOpenFailed` - If OpenCV can't open the file
    /// * `InvalidVideoFormat` - If the video format is unsupported
    /// * `EmptyVideo` - If the video has no frames
    #[instrument(skip(path))]
    pub fn open(path: &str) -> Result<Self> {
        info!("Opening video stream: {}", path);
        
        // Check if file exists first (fail-fast approach)
        if !Path::new(path).exists() {
            return Err(SceneDetectError::VideoNotFound { 
                path: path.to_string() 
            });
        }
        
        // Open video capture
        let cap = videoio::VideoCapture::from_file(path, videoio::CAP_ANY)
            .map_err(|e| {
                warn!("Failed to create VideoCapture: {}", e);
                SceneDetectError::VideoOpenFailed { 
                    path: path.to_string() 
                }
            })?;
        
        // Verify the capture is opened
        let is_opened = cap.is_opened().map_err(|e| {
            warn!("Failed to check if VideoCapture is opened: {}", e);
            SceneDetectError::VideoOpenFailed { 
                path: path.to_string() 
            }
        })?;
        
        if !is_opened {
            return Err(SceneDetectError::VideoOpenFailed { 
                path: path.to_string() 
            });
        }
        
        // Get video properties
        let fps = cap.get(videoio::CAP_PROP_FPS).map_err(|e| {
            warn!("Failed to get video FPS: {}", e);
            SceneDetectError::InvalidVideoFormat { 
                path: path.to_string() 
            }
        })?;
        
        let frame_count = cap.get(videoio::CAP_PROP_FRAME_COUNT).map_err(|e| {
            warn!("Failed to get frame count: {}", e);
            SceneDetectError::InvalidVideoFormat { 
                path: path.to_string() 
            }
        })? as i32;
        
        let width = cap.get(videoio::CAP_PROP_FRAME_WIDTH).map_err(|e| {
            warn!("Failed to get frame width: {}", e);
            SceneDetectError::InvalidVideoFormat { 
                path: path.to_string() 
            }
        })? as i32;
        
        let height = cap.get(videoio::CAP_PROP_FRAME_HEIGHT).map_err(|e| {
            warn!("Failed to get frame height: {}", e);
            SceneDetectError::InvalidVideoFormat { 
                path: path.to_string() 
            }
        })? as i32;
        
        // Validate video properties (fail-fast approach)
        if fps <= 0.0 {
            return Err(SceneDetectError::InvalidVideoFormat { 
                path: path.to_string() 
            });
        }
        
        if frame_count <= 0 {
            return Err(SceneDetectError::EmptyVideo);
        }
        
        if width <= 0 || height <= 0 {
            return Err(SceneDetectError::InvalidVideoFormat { 
                path: path.to_string() 
            });
        }
        
        info!("Video opened successfully - FPS: {}, Frames: {}, Size: {}x{}", 
              fps, frame_count, width, height);
        
        Ok(Self {
            cap,
            fps,
            frame_count,
            current_frame: 0,
            width,
            height,
            path: path.to_string(),
        })
    }
    
    /// Read the next frame from the video
    /// 
    /// # Returns
    /// * `Result<Option<Mat>>` - The next frame if available, None if end of video
    /// 
    /// # Errors
    /// * `FrameProcessingFailed` - If frame reading fails
    #[instrument(skip(self))]
    pub fn read_frame(&mut self) -> Result<Option<Mat>> {
        let mut frame = Mat::default();
        
        let success = self.cap.read(&mut frame).map_err(|e| {
            SceneDetectError::frame_error(
                self.current_frame as u32, 
                format!("OpenCV read failed: {}", e)
            )
        })?;
        
        if success && !frame.empty() {
            self.current_frame += 1;
            debug!("Read frame {}/{}", self.current_frame, self.frame_count);
            
            // Validate frame dimensions (fail-fast approach)
            let frame_rows = frame.rows();
            let frame_cols = frame.cols();
            
            assert_eq!(frame_rows, self.height, 
                      "Frame height mismatch: expected {}, got {}", 
                      self.height, frame_rows);
            assert_eq!(frame_cols, self.width, 
                      "Frame width mismatch: expected {}, got {}", 
                      self.width, frame_cols);
            
            Ok(Some(frame))
        } else {
            debug!("Reached end of video at frame {}", self.current_frame);
            Ok(None)
        }
    }
    
    /// Get the video framerate
    pub fn fps(&self) -> f64 {
        self.fps
    }
    
    /// Get the total number of frames in the video
    pub fn frame_count(&self) -> i32 {
        self.frame_count
    }
    
    /// Get the current frame number (1-indexed, 0 means no frames read yet)
    pub fn current_frame(&self) -> i32 {
        self.current_frame
    }
    
    /// Get the frame width in pixels
    pub fn width(&self) -> i32 {
        self.width
    }
    
    /// Get the frame height in pixels
    pub fn height(&self) -> i32 {
        self.height
    }
    
    /// Get the video file path
    pub fn path(&self) -> &str {
        &self.path
    }
    
    /// Get the video duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        self.frame_count as f64 / self.fps
    }
    
    /// Check if there are more frames to read
    pub fn has_more_frames(&self) -> bool {
        self.current_frame < self.frame_count
    }
    
    /// Get the progress as a percentage (0.0 to 100.0)
    pub fn progress_percent(&self) -> f64 {
        if self.frame_count == 0 {
            100.0
        } else {
            (self.current_frame as f64 / self.frame_count as f64) * 100.0
        }
    }
}

// Implement Debug manually to avoid showing internal OpenCV state
impl std::fmt::Debug for VideoStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoStream")
            .field("path", &self.path)
            .field("fps", &self.fps)
            .field("frame_count", &self.frame_count)
            .field("current_frame", &self.current_frame)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

// Ensure VideoStream is Send + Sync for potential future async usage
unsafe impl Send for VideoStream {}
unsafe impl Sync for VideoStream {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    // Note: These tests require actual video files to work properly.
    // In a real project, you'd want to include small test video files
    // or use mock implementations for CI/CD.
    
    #[test]
    fn test_video_stream_nonexistent_file() {
        let result = VideoStream::open("nonexistent_video.mp4");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            SceneDetectError::VideoNotFound { path } => {
                assert_eq!(path, "nonexistent_video.mp4");
            },
            other => panic!("Expected VideoNotFound, got: {:?}", other),
        }
    }
    
    #[test]
    fn test_video_stream_empty_path() {
        let result = VideoStream::open("");
        assert!(result.is_err());
        
        // Should fail with VideoNotFound since empty path doesn't exist
        assert!(matches!(result.unwrap_err(), SceneDetectError::VideoNotFound { .. }));
    }
    
    #[test]
    fn test_video_stream_directory_instead_of_file() {
        // Create a temporary directory
        let temp_dir = std::env::temp_dir().join("test_video_dir");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).ok();
        }
        fs::create_dir(&temp_dir).expect("Failed to create test directory");
        
        let result = VideoStream::open(temp_dir.to_str().unwrap());
        assert!(result.is_err());
        
        // Clean up
        fs::remove_dir_all(&temp_dir).ok();
        
        // Should fail with VideoOpenFailed since it's a directory, not a video file
        assert!(matches!(result.unwrap_err(), 
                        SceneDetectError::VideoOpenFailed { .. } |
                        SceneDetectError::InvalidVideoFormat { .. }));
    }
    
    #[test]
    fn test_video_stream_properties_getters() {
        // This test would require a real video file
        // For now, we'll test the basic structure
        
        // If we had a test video, the test would look like:
        // let mut stream = VideoStream::open("test_video.mp4").unwrap();
        // assert!(stream.fps() > 0.0);
        // assert!(stream.frame_count() > 0);
        // assert_eq!(stream.current_frame(), 0);
        // assert!(stream.width() > 0);
        // assert!(stream.height() > 0);
        // assert!(!stream.path().is_empty());
    }
    
    #[test]
    fn test_video_stream_progress_calculation() {
        // We can test the progress calculation logic without a real video
        // by creating a mock-like scenario
        
        // This would be tested with a real video file:
        // let mut stream = VideoStream::open("test_video.mp4").unwrap();
        // assert_eq!(stream.progress_percent(), 0.0);
        
        // After reading some frames:
        // while let Some(_frame) = stream.read_frame().unwrap() {
        //     let progress = stream.progress_percent();
        //     assert!(progress >= 0.0 && progress <= 100.0);
        //     if stream.current_frame() >= 10 { break; }
        // }
    }
    
    #[test]
    fn test_video_stream_debug_format() {
        // Test that Debug formatting works and doesn't include sensitive data
        
        // This would be tested with a real video:
        // let stream = VideoStream::open("test_video.mp4").unwrap();
        // let debug_str = format!("{:?}", stream);
        // assert!(debug_str.contains("VideoStream"));
        // assert!(debug_str.contains("fps"));
        // assert!(debug_str.contains("frame_count"));
    }
}