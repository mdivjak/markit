# System Patterns

This file documents recurring patterns and standards used in the project.
It is optional, but recommended to be updated as the project evolves.
2025-06-14 22:42:11 - System patterns documentation initialized.

## Coding Patterns

* **Fail-Fast Approach**: Extensive use of assertions throughout the application for early bug detection
* **Comprehensive Testing**: Unit tests required for all functionality with high coverage expectations
* **Modular Design**: Clear separation between core logic, GUI, and utility functions
* **Incremental Development**: Small, reviewable changes with single-file focus when possible

## Architectural Patterns

* **Layered Architecture**: 
  - Core layer: Scene detection algorithms, video processing, MIDI generation
  - GUI layer: User interface and interaction handling
  - Testing layer: Comprehensive unit and integration tests
* **Reference Implementation Pattern**: Maintain PySceneDetect example for comparison and learning
* **Cross-Platform Compatibility**: Python-based implementation for broad platform support

## Testing Patterns

* **Unit Test Coverage**: All functions and methods must have corresponding unit tests
* **Assertion-Heavy Code**: Use assertions liberally to catch issues early in development
* **Test-Driven Development**: Write tests before or alongside implementation
* **Incremental Testing**: Test each small change independently before integration

## Development Patterns (Rust Migration Considerations)

* **Module Structure**: Use `.rs` files with same-named folders instead of `mod.rs` files
* **Instrumentation**: Extensive use of tokio tracing crate for logging and debugging
* **Error Handling**: Robust error handling with proper error propagation
* **Micro-Changes**: Prioritize small, focused changes for easy review and validation