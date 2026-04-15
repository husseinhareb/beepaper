//! Native Wayland wallpaper application entry points.

pub mod app;
pub mod buffer;
pub mod globals;
pub mod layer;
pub mod output;
pub mod render;

use std::path::Path;

use crate::config::ApplyMode;
use crate::error::WaylandError;

pub use app::PreparedWallpaper;

/// Options controlling native wallpaper application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplyOptions {
    /// How the image should be scaled to the target output.
    pub mode: ApplyMode,
}

/// Prepare a native Wayland wallpaper surface and return a running handle.
pub fn prepare_wallpaper(
    path: &Path,
    options: &ApplyOptions,
) -> Result<PreparedWallpaper, WaylandError> {
    app::WaylandApp::prepare(path, options)
}

/// Apply a wallpaper natively on Wayland and keep the process alive while it is shown.
pub fn apply_wallpaper(path: &Path, options: &ApplyOptions) -> Result<(), WaylandError> {
    prepare_wallpaper(path, options)?.run()
}
