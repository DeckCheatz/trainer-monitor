# Agent Guide: Trainer Monitor

This document provides essential context for AI agents working on the `trainer-monitor` project.

## Project Overview

`trainer-monitor` is a lightweight Rust-based CLI utility designed to manage the lifecycle of a game and its associated trainer. It ensures both processes are launched and waits for both to terminate before exiting.

### Key Functionality

- Spawns a game executable provided as a command-line argument.
- Spawns a pre-defined trainer executable (currently hardcoded as `Wand.exe`).
- Monitors both processes in a loop until both have exited.
- Uses `anyhow` for error handling and `thiserror` for custom error types (though not yet heavily utilized).

## Technical Stack

- **Language:** Rust (Edition 2024)
- **Target OS:** Windows (due to hardcoded paths and executable naming conventions).
- **Core Dependencies:**
  - `anyhow`: Flexible error handling.
  - `thiserror`: (Included for future custom error definitions).

## Project Structure

- `Cargo.toml`: Project configuration and dependencies.
- `src/main.rs`: Primary entry point and process management logic.

## Conventions & Development

### Implementation Details

- **Hardcoded Paths:** The trainer path is currently hardcoded in `src/main.rs` as `C:\users\steamuser\AppData\Roaming\Wand\Wand.exe`.
- **Process Management:** Uses `std::process::Command` to spawn processes and `try_wait()` to monitor them without blocking.
- **Polling Loop:** A simple loop with a 100ms sleep prevents high CPU usage while waiting for processes to exit.

### Future Improvements (Awaiting Directives)

- Making the trainer path configurable (via config file or environment variables).
- Supporting non-Windows environments.
- Enhanced logging and error reporting.

## Workflow

1.  **Research:** Examine `src/main.rs` for process spawning logic.
2.  **Execution:** Use `cargo build` to compile and `cargo run -- <path-to-game>` for testing.
3.  **Validation:** Ensure both the game and the mock/actual trainer are handled correctly.
