//! Domain errors for `wallselect`.

use thiserror::Error;

/// Errors related to resolving application directories.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The platform did not provide usable project directories.
    #[error("unable to determine application config and state directories")]
    ProjectDirsUnavailable,
}

/// Errors related to wallpaper selection.
#[derive(Debug, Error)]
pub enum SelectionError {
    /// No wallpaper candidates were available for selection.
    #[error("no wallpaper candidates available")]
    NoCandidates,
}
