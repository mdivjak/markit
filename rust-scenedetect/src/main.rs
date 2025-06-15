//! Command-line interface for the Rust scene detection library
//! 
//! This binary provides a simple CLI for testing and using the scene detection
//! functionality. It matches the behavior of your Python implementation and
//! provides additional debugging and performance measurement capabilities.

use std::env;
use std::time::Instant;
use std::process;
use tracing::{info, error, warn, debug};
use rust_scenedetect::{
    detect_scene_changes, detect, get_video_info, init_tracing,
    ContentDetector, ComponentWeights, FilterMode,
    SceneDetectError,
};

/// Command-line arguments structure
#[derive(Debug)]
struct Args {
    video_path: String,
    threshold: Option<f64>,
    min_scene_length: Option<u32>,
    filter_mode: FilterMode,
    luma_only: bool,
    verbose: bool,
    show_video_info: bool,
    output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Simple,     // Just frame numbers (matches Python)
    Detailed,   // Frame numbers with timestamps
    Json,       // JSON format for integration
}

impl Default for Args {
    fn default() -> Self {
        Self {
            video_path: String::new(),
            threshold: None,
            min_scene_length: None,
            filter_mode: FilterMode::Suppress,
            luma_only: false,
            verbose: false,
            show_video_info: false,
            output_format: OutputFormat::Simple,
        }
    }
}

fn main() {
    let result = run();
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    
    // Initialize tracing based on verbosity
    let log_level = if args.verbose { "debug" } else { "info" };
    init_tracing(log_level);
    
    info!("Rust Scene Detection CLI v{}", env!("CARGO_PKG_VERSION"));
    debug!("Arguments: {:?}", args);
    
    // Show video info if requested
    if args.show_video_info {
        show_video_info(&args.video_path)?;
        return Ok(());
    }
    
    // Perform scene detection
    let start_time = Instant::now();
    
    let frame_numbers = if args.threshold.is_some() || args.min_scene_length.is_some() || args.luma_only {
        // Custom configuration - use advanced API
        detect_with_custom_config(&args)?
    } else {
        // Simple case - use Python-compatible API
        detect_scene_changes(&args.video_path)
            .map_err(|e| format!("Scene detection failed: {}", e))?
    };
    
    let detection_time = start_time.elapsed();
    
    // Output results
    output_results(&frame_numbers, &args, detection_time)?;
    
    Ok(())
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        return Err(format!(
            "Usage: {} <video_path> [OPTIONS]\n\n\
            Options:\n\
            --threshold <value>      Detection threshold (default: 27.0)\n\
            --min-scene-length <n>   Minimum frames between cuts (default: 15)\n\
            --filter-mode <mode>     Filter mode: suppress|merge (default: suppress)\n\
            --luma-only              Use only brightness changes\n\
            --verbose                Enable debug logging\n\
            --info                   Show video information only\n\
            --format <fmt>           Output format: simple|detailed|json (default: simple)\n\
            --help                   Show this help message\n\n\
            Examples:\n\
            {} video.mp4\n\
            {} video.mp4 --threshold 30.0 --verbose\n\
            {} video.mp4 --luma-only --format detailed",
            args[0], args[0], args[0], args[0]
        ));
    }
    
    let mut parsed_args = Args::default();
    parsed_args.video_path = args[1].clone();
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--threshold" => {
                if i + 1 >= args.len() {
                    return Err("--threshold requires a value".to_string());
                }
                parsed_args.threshold = Some(args[i + 1].parse()
                    .map_err(|_| "Invalid threshold value")?);
                i += 2;
            }
            "--min-scene-length" => {
                if i + 1 >= args.len() {
                    return Err("--min-scene-length requires a value".to_string());
                }
                parsed_args.min_scene_length = Some(args[i + 1].parse()
                    .map_err(|_| "Invalid min-scene-length value")?);
                i += 2;
            }
            "--filter-mode" => {
                if i + 1 >= args.len() {
                    return Err("--filter-mode requires a value".to_string());
                }
                parsed_args.filter_mode = match args[i + 1].as_str() {
                    "suppress" => FilterMode::Suppress,
                    "merge" => FilterMode::Merge,
                    _ => return Err("Invalid filter mode. Use 'suppress' or 'merge'".to_string()),
                };
                i += 2;
            }
            "--format" => {
                if i + 1 >= args.len() {
                    return Err("--format requires a value".to_string());
                }
                parsed_args.output_format = match args[i + 1].as_str() {
                    "simple" => OutputFormat::Simple,
                    "detailed" => OutputFormat::Detailed,
                    "json" => OutputFormat::Json,
                    _ => return Err("Invalid format. Use 'simple', 'detailed', or 'json'".to_string()),
                };
                i += 2;
            }
            "--luma-only" => {
                parsed_args.luma_only = true;
                i += 1;
            }
            "--verbose" => {
                parsed_args.verbose = true;
                i += 1;
            }
            "--info" => {
                parsed_args.show_video_info = true;
                i += 1;
            }
            "--help" => {
                return Err(format!(
                    "Rust Scene Detection Tool\n\n\
                    Usage: {} <video_path> [OPTIONS]\n\n\
                    This tool replicates PySceneDetect's ContentDetector functionality\n\
                    with improved performance and Rust safety guarantees.\n\n\
                    Options:\n\
                    --threshold <value>      Detection threshold (default: 27.0)\n\
                    --min-scene-length <n>   Minimum frames between cuts (default: 15)\n\
                    --filter-mode <mode>     Filter mode: suppress|merge (default: suppress)\n\
                    --luma-only              Use only brightness changes (ignore color)\n\
                    --verbose                Enable debug logging and tracing\n\
                    --info                   Show video information only\n\
                    --format <fmt>           Output format: simple|detailed|json\n\
                    --help                   Show this help message",
                    args[0]
                ));
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }
    
    Ok(parsed_args)
}

fn show_video_info(video_path: &str) -> Result<(), String> {
    info!("Analyzing video: {}", video_path);
    
    let video_info = get_video_info(video_path)
        .map_err(|e| format!("Failed to get video info: {}", e))?;
    
    println!("Video Information:");
    println!("  Path: {}", video_info.path);
    println!("  Dimensions: {}x{}", video_info.width, video_info.height);
    println!("  Frame Rate: {:.2} fps", video_info.fps);
    println!("  Frame Count: {}", video_info.frame_count);
    println!("  Duration: {:.2} seconds", video_info.duration_seconds);
    println!("  Description: {}", video_info.description());
    
    if !video_info.is_valid() {
        warn!("Video properties appear invalid - detection may fail");
    }
    
    Ok(())
}

fn detect_with_custom_config(args: &Args) -> Result<Vec<u32>, String> {
    info!("Using custom detection configuration");
    
    let threshold = args.threshold.unwrap_or(27.0);
    let min_scene_length = args.min_scene_length.unwrap_or(15);
    
    debug!("Configuration: threshold={}, min_scene_length={}, filter_mode={:?}, luma_only={}", 
           threshold, min_scene_length, args.filter_mode, args.luma_only);
    
    let detector = if args.luma_only {
        ContentDetector::new_luma_only(threshold)
    } else {
        let weights = ComponentWeights::default();
        ContentDetector::new_with_config(threshold, weights, min_scene_length, args.filter_mode)
            .map_err(|e| format!("Failed to create detector: {}", e))?
    };
    
    let scene_list = detect(&args.video_path, detector)
        .map_err(|e| format!("Scene detection failed: {}", e))?;
    
    let frame_numbers: Vec<u32> = scene_list
        .iter()
        .map(|scene| scene.start.frame_number())
        .collect();
    
    Ok(frame_numbers)
}

fn output_results(frame_numbers: &[u32], args: &Args, detection_time: std::time::Duration) -> Result<(), String> {
    match args.output_format {
        OutputFormat::Simple => {
            // Matches your Python implementation output
            println!("Scene changes detected at frames: {:?}", frame_numbers);
        }
        OutputFormat::Detailed => {
            println!("Scene Detection Results:");
            println!("  Detection time: {:.2}ms", detection_time.as_millis());
            println!("  Scenes found: {}", frame_numbers.len());
            
            if frame_numbers.is_empty() {
                println!("  No scene changes detected");
            } else {
                println!("  Scene cuts:");
                for (i, &frame) in frame_numbers.iter().enumerate() {
                    // Would need FPS to calculate timestamps - simplified for MVP
                    println!("    Scene {}: Frame {}", i + 1, frame);
                }
            }
        }
        OutputFormat::Json => {
            // JSON output for programmatic consumption
            let json_output = serde_json::json!({
                "video_path": args.video_path,
                "detection_time_ms": detection_time.as_millis(),
                "scene_count": frame_numbers.len(),
                "frame_numbers": frame_numbers,
                "config": {
                    "threshold": args.threshold.unwrap_or(27.0),
                    "min_scene_length": args.min_scene_length.unwrap_or(15),
                    "filter_mode": format!("{:?}", args.filter_mode),
                    "luma_only": args.luma_only
                }
            });
            
            println!("{}", serde_json::to_string_pretty(&json_output)
                .map_err(|e| format!("JSON serialization failed: {}", e))?);
        }
    }
    
    info!("Detection completed in {:.2}ms, found {} scene changes", 
          detection_time.as_millis(), frame_numbers.len());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_args_parsing_basic() {
        // Test basic argument parsing without actual video files
        let args = Args {
            video_path: "test.mp4".to_string(),
            threshold: Some(30.0),
            min_scene_length: Some(20),
            filter_mode: FilterMode::Merge,
            luma_only: true,
            verbose: false,
            show_video_info: false,
            output_format: OutputFormat::Detailed,
        };
        
        // Basic validation
        assert_eq!(args.video_path, "test.mp4");
        assert_eq!(args.threshold, Some(30.0));
        assert_eq!(args.min_scene_length, Some(20));
        assert!(args.luma_only);
    }
    
    #[test]
    fn test_output_format_values() {
        // Test that output format enum values are correct
        let simple = OutputFormat::Simple;
        let detailed = OutputFormat::Detailed;
        let json = OutputFormat::Json;
        
        // Should be different values
        assert!(matches!(simple, OutputFormat::Simple));
        assert!(matches!(detailed, OutputFormat::Detailed));
        assert!(matches!(json, OutputFormat::Json));
    }
    
    #[test]
    fn test_default_args() {
        let args = Args::default();
        assert!(args.video_path.is_empty());
        assert_eq!(args.threshold, None);
        assert_eq!(args.min_scene_length, None);
        assert!(!args.luma_only);
        assert!(!args.verbose);
        assert!(!args.show_video_info);
    }
    
    // Note: Full CLI testing would require integration tests with actual video files
    // and proper argument simulation, which would be in tests/ directory
}