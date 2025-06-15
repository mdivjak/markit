# Rust Scene Detection Library

A minimal but performant Rust implementation of PySceneDetect's ContentDetector functionality. This library provides video scene detection capabilities with improved performance and Rust safety guarantees.

## Features

- **HSV-based Scene Detection**: Implements PySceneDetect's ContentDetector algorithm
- **Drop-in Python Replacement**: Compatible API with your existing Python usage
- **Comprehensive Error Handling**: Robust error handling with detailed error messages
- **Performance Optimized**: Built for speed with Rust's zero-cost abstractions
- **Extensive Instrumentation**: Built-in tracing for debugging and performance analysis
- **Flash Filtering**: Prevents false positives from brief flashes or transitions
- **Flexible Configuration**: Customizable thresholds, weights, and filtering modes

## Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rust-scenedetect = "0.1.0"
```

### Basic Usage

```rust
use rust_scenedetect::detect_scene_changes;

// Drop-in replacement for your Python function
let frame_numbers = detect_scene_changes("video.mp4")?;
println!("Scene changes at frames: {:?}", frame_numbers);
```

### Advanced Usage

```rust
use rust_scenedetect::{ContentDetector, detect, ComponentWeights, FilterMode};

// Custom detector configuration
let detector = ContentDetector::new_with_config(
    30.0,                              // threshold
    ComponentWeights::default(),       // HSV weights
    20,                               // min scene length
    FilterMode::Merge                 // filter mode
)?;

let scenes = detect("video.mp4", detector)?;
for (i, scene) in scenes.iter().enumerate() {
    println!("Scene {}: frames {} to {}", 
        i + 1, 
        scene.start.frame_number(),
        scene.end.as_ref().map_or("end".to_string(), |e| e.frame_number().to_string())
    );
}
```

## Command Line Interface

The library includes a CLI tool for testing and batch processing:

```bash
# Basic usage (matches your Python implementation)
cargo run --bin rust-scenedetect video.mp4

# Custom threshold
cargo run --bin rust-scenedetect video.mp4 --threshold 30.0

# Luma-only detection (brightness changes only)
cargo run --bin rust-scenedetect video.mp4 --luma-only

# Detailed output with performance metrics
cargo run --bin rust-scenedetect video.mp4 --format detailed --verbose

# JSON output for integration
cargo run --bin rust-scenedetect video.mp4 --format json

# Show video information
cargo run --bin rust-scenedetect video.mp4 --info
```

## Architecture

The library follows a modular design with clear separation of concerns:

```
rust-scenedetect/
├── src/
│   ├── lib.rs              # Public API
│   ├── main.rs             # CLI binary
│   ├── common.rs           # Shared types and utilities
│   ├── video_stream.rs     # OpenCV video reading
│   ├── content_detector.rs # Core detection algorithm
│   └── flash_filter.rs     # Scene length filtering
├── tests/
│   └── integration_tests.rs # Integration tests
└── docs/
    └── rust-mvp-implementation-plan.md # Detailed design
```

### Core Components

1. **VideoStream**: OpenCV-based video reading with error handling
2. **ContentDetector**: HSV-based scene change detection
3. **FlashFilter**: Minimum scene length enforcement
4. **Common Types**: FrameTimecode, SceneCut, comprehensive error types

## API Reference

### Main Functions

#### `detect_scene_changes(video_path: &str) -> Result<Vec<u32>>`

Simple function that matches your Python implementation exactly. Returns frame numbers where scene changes occur.

```rust
let frame_numbers = detect_scene_changes("video.mp4")?;
```

#### `detect(video_path: &str, detector: ContentDetector) -> Result<Vec<SceneCut>>`

Advanced function with custom detector configuration. Returns detailed scene information.

```rust
let detector = ContentDetector::new(27.0);
let scenes = detect("video.mp4", detector)?;
```

#### `get_video_info(video_path: &str) -> Result<VideoInfo>`

Get video metadata without performing detection.

```rust
let info = get_video_info("video.mp4")?;
println!("Video: {}", info.description());
```

### Configuration Types

#### `ContentDetector`

Main detection class with multiple constructors:

- `ContentDetector::new(threshold: f64)` - Basic detector
- `ContentDetector::new_luma_only(threshold: f64)` - Brightness-only detection
- `ContentDetector::new_with_config(...)` - Full customization

#### `ComponentWeights`

Controls how much each color channel contributes to the detection score:

```rust
let weights = ComponentWeights {
    delta_hue: 1.0,     // Hue changes
    delta_sat: 1.0,     // Saturation changes  
    delta_lum: 1.0,     // Luminance changes
    delta_edges: 0.0,   // Edge changes (disabled in MVP)
};
```

#### `FilterMode`

Controls how consecutive scene cuts are handled:

- `FilterMode::Suppress` - Suppress cuts until minimum length passes (default)
- `FilterMode::Merge` - Merge consecutive cuts into single cut

## Performance Comparison

The Rust implementation is designed for performance measurement against your Python version:

| Metric | Python (baseline) | Rust (target) |
|--------|------------------|---------------|
| Processing Speed | 1x | 2-5x faster |
| Memory Usage | baseline | 30-50% less |
| CPU Usage | baseline | 20-40% less |

*Actual performance will vary based on video characteristics and hardware.*

## Error Handling

The library uses comprehensive error types for robust error handling:

```rust
match detect_scene_changes("video.mp4") {
    Ok(frames) => println!("Found {} scene changes", frames.len()),
    Err(SceneDetectError::VideoNotFound { path }) => {
        eprintln!("Video file not found: {}", path);
    },
    Err(SceneDetectError::VideoOpenFailed { path }) => {
        eprintln!("Could not open video: {}", path);
    },
    Err(e) => eprintln!("Detection failed: {}", e),
}
```

## Development

### Building

```bash
# Build the library
cargo build

# Build with optimizations
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run --bin rust-scenedetect video.mp4
```

### Testing

The library includes comprehensive tests:

```bash
# Unit tests (in each module)
cargo test --lib

# Integration tests
cargo test --test integration_tests

# All tests with verbose output
cargo test -- --nocapture
```

### Benchmarking

Performance benchmarks are included in the test suite:

```bash
# Run benchmark tests
cargo test benchmark

# For detailed benchmarking, add criterion (see Cargo.toml)
cargo bench
```

## Dependencies

- **opencv**: Video processing (matches PySceneDetect's backend)
- **tracing**: Instrumentation and logging
- **anyhow/thiserror**: Error handling
- **serde_json**: JSON output for CLI

## Algorithm Details

### Scene Detection Process

1. **Frame Reading**: Use OpenCV to read video frames sequentially
2. **Color Space Conversion**: Convert BGR frames to HSV color space
3. **Channel Analysis**: Calculate mean pixel differences for H, S, V channels
4. **Score Calculation**: Weighted average of channel differences
5. **Threshold Comparison**: Compare score against detection threshold
6. **Flash Filtering**: Apply minimum scene length constraints
7. **Result Generation**: Output frame numbers and timing information

### HSV Analysis

The ContentDetector analyzes three color channels:

- **Hue**: Color information (what color)
- **Saturation**: Color intensity (how much color)
- **Value/Luminance**: Brightness (how bright)

This approach is more robust than RGB analysis for scene detection because it separates color information from lighting conditions.

### Flash Filtering

Two filtering modes prevent false positives:

1. **Suppress Mode**: Ignore cuts that occur too soon after the previous cut
2. **Merge Mode**: Combine consecutive cuts into a single transition

## Comparison with PySceneDetect

### Similarities

- Same ContentDetector algorithm
- Identical default parameters (threshold 27.0, min scene length 15)
- HSV color space analysis
- Compatible output format

### Differences

- **Performance**: Rust implementation is significantly faster
- **Memory Safety**: No risk of memory leaks or segfaults
- **Error Handling**: More comprehensive error types
- **Instrumentation**: Built-in tracing and debugging
- **Edge Detection**: Simplified for MVP (can be added later)

## Integration with Existing Workflow

This library is designed as a drop-in replacement for your Python scene detection:

### Before (Python)
```python
from my_impl.core.scene_detection import detect_scene_changes
frame_numbers = detect_scene_changes("video.mp4")
```

### After (Rust)
```rust
use rust_scenedetect::detect_scene_changes;
let frame_numbers = detect_scene_changes("video.mp4")?;
```

The output format is identical, so the rest of your pipeline (MIDI generation, etc.) works unchanged.

## Future Enhancements

The MVP focuses on core functionality, but the architecture supports future additions:

- **Edge Detection**: Canny edge detection for improved accuracy
- **Additional Detectors**: Threshold detector, fade detector, etc.
- **GPU Acceleration**: CUDA/OpenCL support for large videos
- **Parallel Processing**: Multi-threaded frame analysis
- **Video Formats**: Support for more exotic video formats
- **Streaming**: Real-time scene detection for live video

## License

MIT License - see LICENSE file for details.

## Contributing

1. Follow the existing code patterns and documentation style
2. Add comprehensive tests for new functionality
3. Use the fail-fast approach with assertions
4. Include tracing instrumentation for debugging
5. Keep changes small and reviewable (one file at a time preferred)

## Troubleshooting

### Common Issues

**OpenCV not found**: Ensure OpenCV is installed and in your system PATH.

**Video won't open**: Check that the video file exists and is in a supported format.

**No scene cuts detected**: Try lowering the threshold value (default is 27.0).

**Too many false positives**: Try increasing the minimum scene length or threshold.

### Debug Mode

Enable detailed logging to troubleshoot issues:

```bash
RUST_LOG=debug cargo run --bin rust-scenedetect video.mp4 --verbose
```

This will show frame-by-frame processing details and performance metrics.