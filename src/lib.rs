#![forbid(unsafe_code)]
//! Library modules for the `beepaper` CLI.

/// Command-line argument parsing.
pub mod cli;
/// Configuration loading, defaults, and path resolution.
pub mod config;
/// Domain-specific error types.
pub mod error;
/// History helpers for trimming and viewing prior selections.
pub mod history;
/// Wallpaper directory scanning and image filtering.
pub mod scanner;
/// Random wallpaper selection logic.
pub mod selector;
/// Persisted scan and selection state.
pub mod state;
/// Native Wayland wallpaper application.
pub mod wayland;
