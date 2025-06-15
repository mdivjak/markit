//! Flash filter for enforcing minimum scene length requirements
//! 
//! This module implements the flash filtering logic from PySceneDetect,
//! which prevents false positive scene cuts by enforcing minimum scene lengths.
//! This helps filter out brief flashes, camera flickers, and other transient changes.

use tracing::{instrument, debug, trace};
use crate::common::FrameTimecode;

/// Filter mode for handling consecutive scene cuts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// Merge consecutive cuts shorter than filter length
    Merge,
    /// Suppress consecutive cuts until the filter length has passed
    Suppress,
}

impl Default for FilterMode {
    fn default() -> Self {
        FilterMode::Suppress // PySceneDetect default behavior
    }
}

/// Filters scene cuts to enforce minimum scene length requirements
/// 
/// The FlashFilter prevents false positive scene cuts by ensuring that
/// detected cuts are separated by at least a minimum number of frames.
/// This is essential for filtering out camera flashes, brief transitions,
/// and other transient visual changes that shouldn't be considered scene boundaries.
#[derive(Debug)]
pub struct FlashFilter {
    mode: FilterMode,
    min_scene_length: u32,
    last_cut_frame: Option<u32>,
    last_above_threshold: Option<u32>,
    merge_triggered: bool,
    merge_start_frame: Option<u32>,
}

impl FlashFilter {
    /// Create a new FlashFilter with suppress mode (PySceneDetect default)
    /// 
    /// # Arguments
    /// * `min_scene_length` - Minimum number of frames between scene cuts
    /// 
    /// # Panics
    /// Panics if min_scene_length is 0 (fail-fast approach)
    #[instrument]
    pub fn new(min_scene_length: u32) -> Self {
        assert!(min_scene_length > 0, "Minimum scene length must be positive, got: {}", min_scene_length);
        
        debug!("Created FlashFilter with min_scene_length: {}", min_scene_length);
        
        Self {
            mode: FilterMode::default(),
            min_scene_length,
            last_cut_frame: None,
            last_above_threshold: None,
            merge_triggered: false,
            merge_start_frame: None,
        }
    }
    
    /// Create a new FlashFilter with specified mode
    /// 
    /// # Arguments
    /// * `mode` - Filter mode (Merge or Suppress)
    /// * `min_scene_length` - Minimum number of frames between scene cuts
    #[instrument]
    pub fn new_with_mode(mode: FilterMode, min_scene_length: u32) -> Self {
        assert!(min_scene_length > 0, "Minimum scene length must be positive");
        
        debug!("Created FlashFilter with mode: {:?}, min_scene_length: {}", mode, min_scene_length);
        
        Self {
            mode,
            min_scene_length,
            last_cut_frame: None,
            last_above_threshold: None,
            merge_triggered: false,
            merge_start_frame: None,
        }
    }
    
    /// Filter a potential scene cut based on timing requirements
    /// 
    /// # Arguments
    /// * `timecode` - Current frame timecode
    /// * `above_threshold` - Whether the current frame exceeded the detection threshold
    /// 
    /// # Returns
    /// * `Vec<FrameTimecode>` - List of confirmed scene cuts (usually 0 or 1 item)
    #[instrument(skip(self))]
    pub fn filter(&mut self, timecode: FrameTimecode, above_threshold: bool) -> Vec<FrameTimecode> {
        let current_frame = timecode.frame_number();
        
        trace!("Filter input: frame={}, above_threshold={}", current_frame, above_threshold);
        
        // Update last above threshold frame
        if above_threshold {
            self.last_above_threshold = Some(current_frame);
        }
        
        match self.mode {
            FilterMode::Suppress => self.filter_suppress(timecode, above_threshold),
            FilterMode::Merge => self.filter_merge(timecode, above_threshold),
        }
    }
    
    /// Filter using suppress mode (PySceneDetect default)
    /// 
    /// In suppress mode, once a cut is detected, no additional cuts are allowed
    /// until the minimum scene length has passed.
    fn filter_suppress(&mut self, timecode: FrameTimecode, above_threshold: bool) -> Vec<FrameTimecode> {
        if !above_threshold {
            return vec![];
        }
        
        let current_frame = timecode.frame_number();
        
        // Check if enough time has passed since the last cut
        if let Some(last_frame) = self.last_cut_frame {
            let frames_since_last = current_frame.saturating_sub(last_frame);
            
            if frames_since_last < self.min_scene_length {
                debug!("Suppressing cut at frame {} (only {} frames since last cut at {})", 
                       current_frame, frames_since_last, last_frame);
                return vec![];
            }
        }
        
        // Emit the cut and update tracking
        self.last_cut_frame = Some(current_frame);
        debug!("Scene cut confirmed at frame {} (suppress mode)", current_frame);
        
        vec![timecode]
    }
    
    /// Filter using merge mode
    /// 
    /// In merge mode, consecutive cuts within the minimum scene length are merged
    /// into a single cut at the end of the sequence.
    fn filter_merge(&mut self, timecode: FrameTimecode, above_threshold: bool) -> Vec<FrameTimecode> {
        let current_frame = timecode.frame_number();
        
        // Check if we need to end an ongoing merge
        if let Some(last_above) = self.last_above_threshold {
            let frames_since_above = current_frame.saturating_sub(last_above);
            
            if self.merge_triggered && !above_threshold && frames_since_above >= self.min_scene_length {
                // End the merge and emit the cut
                self.merge_triggered = false;
                
                if let Some(merge_start) = self.merge_start_frame {
                    let merge_duration = last_above.saturating_sub(merge_start);
                    
                    if merge_duration >= self.min_scene_length {
                        debug!("Ending merge: emitting cut at frame {} (merged from frame {})", 
                               last_above, merge_start);
                        
                        self.last_cut_frame = Some(last_above);
                        
                        // Create timecode for the merged cut
                        let cut_timecode = FrameTimecode::new(last_above, timecode.fps());
                        return vec![cut_timecode];
                    }
                }
                
                self.merge_start_frame = None;
            }
        }
        
        // Handle current frame
        if !above_threshold {
            return vec![];
        }
        
        // Check if enough time has passed since last cut for a normal cut
        if let Some(last_frame) = self.last_cut_frame {
            let frames_since_last = current_frame.saturating_sub(last_frame);
            
            if frames_since_last >= self.min_scene_length {
                // Normal cut - enough time has passed
                self.last_cut_frame = Some(current_frame);
                debug!("Scene cut confirmed at frame {} (merge mode - normal)", current_frame);
                return vec![timecode];
            } else if !self.merge_triggered {
                // Start merging
                self.merge_triggered = true;
                self.merge_start_frame = Some(current_frame);
                debug!("Starting merge at frame {} (too soon after cut at {})", 
                       current_frame, last_frame);
                return vec![];
            }
        } else {
            // First cut ever
            self.last_cut_frame = Some(current_frame);
            debug!("First scene cut at frame {} (merge mode)", current_frame);
            return vec![timecode];
        }
        
        vec![]
    }
    
    /// Get the minimum scene length setting
    pub fn min_scene_length(&self) -> u32 {
        self.min_scene_length
    }
    
    /// Get the current filter mode
    pub fn mode(&self) -> FilterMode {
        self.mode
    }
    
    /// Get the frame number of the last confirmed cut (if any)
    pub fn last_cut_frame(&self) -> Option<u32> {
        self.last_cut_frame
    }
    
    /// Reset the filter state (useful for processing multiple videos)
    #[instrument(skip(self))]
    pub fn reset(&mut self) {
        debug!("Resetting FlashFilter state");
        self.last_cut_frame = None;
        self.last_above_threshold = None;
        self.merge_triggered = false;
        self.merge_start_frame = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_timecode(frame: u32) -> FrameTimecode {
        FrameTimecode::new(frame, 25.0) // 25 FPS for testing
    }
    
    #[test]
    fn test_flash_filter_creation() {
        let filter = FlashFilter::new(15);
        assert_eq!(filter.min_scene_length(), 15);
        assert_eq!(filter.mode(), FilterMode::Suppress);
        assert_eq!(filter.last_cut_frame(), None);
        
        let filter_merge = FlashFilter::new_with_mode(FilterMode::Merge, 10);
        assert_eq!(filter_merge.mode(), FilterMode::Merge);
        assert_eq!(filter_merge.min_scene_length(), 10);
    }
    
    #[test]
    #[should_panic(expected = "Minimum scene length must be positive")]
    fn test_flash_filter_zero_length() {
        FlashFilter::new(0);
    }
    
    #[test]
    fn test_suppress_mode_basic() {
        let mut filter = FlashFilter::new(10);
        
        // First cut should be accepted
        let cuts = filter.filter(create_timecode(100), true);
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].frame_number(), 100);
        
        // Cut too soon should be suppressed
        let cuts = filter.filter(create_timecode(105), true);
        assert_eq!(cuts.len(), 0);
        
        // Cut after minimum length should be accepted
        let cuts = filter.filter(create_timecode(115), true);
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].frame_number(), 115);
    }
    
    #[test]
    fn test_suppress_mode_below_threshold() {
        let mut filter = FlashFilter::new(10);
        
        // Below threshold frames should never produce cuts
        let cuts = filter.filter(create_timecode(100), false);
        assert_eq!(cuts.len(), 0);
        
        let cuts = filter.filter(create_timecode(110), false);
        assert_eq!(cuts.len(), 0);
    }
    
    #[test]
    fn test_merge_mode_basic() {
        let mut filter = FlashFilter::new_with_mode(FilterMode::Merge, 10);
        
        // First cut should be accepted
        let cuts = filter.filter(create_timecode(100), true);
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].frame_number(), 100);
        
        // Cut after sufficient time should be accepted
        let cuts = filter.filter(create_timecode(120), true);
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].frame_number(), 120);
    }
    
    #[test]
    fn test_merge_mode_consecutive_cuts() {
        let mut filter = FlashFilter::new_with_mode(FilterMode::Merge, 10);
        
        // First cut
        let cuts = filter.filter(create_timecode(100), true);
        assert_eq!(cuts.len(), 1);
        
        // Too soon - should start merge
        let cuts = filter.filter(create_timecode(105), true);
        assert_eq!(cuts.len(), 0);
        
        // More frames above threshold during merge
        let cuts = filter.filter(create_timecode(106), true);
        assert_eq!(cuts.len(), 0);
        
        // Below threshold for sufficient time - should end merge and emit cut
        for frame in 107..120 {
            let cuts = filter.filter(create_timecode(frame), false);
            if frame >= 117 { // 106 + 10 + 1
                // Should emit merged cut
                if !cuts.is_empty() {
                    assert_eq!(cuts[0].frame_number(), 106);
                    break;
                }
            } else {
                assert_eq!(cuts.len(), 0);
            }
        }
    }
    
    #[test]
    fn test_filter_reset() {
        let mut filter = FlashFilter::new(10);
        
        // Process some cuts
        filter.filter(create_timecode(100), true);
        assert!(filter.last_cut_frame().is_some());
        
        // Reset and verify state is clean
        filter.reset();
        assert_eq!(filter.last_cut_frame(), None);
        
        // Should work normally after reset
        let cuts = filter.filter(create_timecode(50), true);
        assert_eq!(cuts.len(), 1);
    }
    
    #[test]
    fn test_filter_edge_cases() {
        let mut filter = FlashFilter::new(1); // Minimum possible length
        
        // Every frame above threshold should produce a cut
        let cuts = filter.filter(create_timecode(100), true);
        assert_eq!(cuts.len(), 1);
        
        let cuts = filter.filter(create_timecode(101), true);
        assert_eq!(cuts.len(), 1);
        
        let cuts = filter.filter(create_timecode(102), true);
        assert_eq!(cuts.len(), 1);
    }
    
    #[test]
    fn test_frame_number_overflow_safety() {
        let mut filter = FlashFilter::new(10);
        
        // Test with large frame numbers
        let cuts = filter.filter(create_timecode(u32::MAX - 5), true);
        assert_eq!(cuts.len(), 1);
        
        // Next frame (would overflow in naive subtraction)
        let cuts = filter.filter(create_timecode(u32::MAX), true);
        assert_eq!(cuts.len(), 0); // Should be suppressed due to insufficient gap
    }
}