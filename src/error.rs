//! Domain errors for `beepaper`.

use thiserror::Error;
use wayland_client::{ConnectError, DispatchError, globals::GlobalError};

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

/// Errors related to the native Wayland wallpaper path.
#[derive(Debug, Error)]
pub enum WaylandError {
    /// Connecting to the Wayland compositor failed.
    #[error("failed to connect to the Wayland compositor")]
    Connect(#[from] ConnectError),
    /// Initial global discovery failed.
    #[error("failed to initialize Wayland globals")]
    GlobalInit(#[from] GlobalError),
    /// Dispatching the event queue failed.
    #[error("failed to dispatch Wayland events")]
    Dispatch(#[from] DispatchError),
    /// Reading or writing the shared-memory backing file failed.
    #[error("wayland shared-memory I/O failed")]
    Io(#[from] std::io::Error),
    /// Decoding the wallpaper image failed.
    #[error("failed to decode wallpaper image")]
    Image(#[from] image::ImageError),
    /// A required global was not advertised by the compositor.
    #[error("required Wayland global `{0}` is not available")]
    MissingGlobal(&'static str),
    /// A required global was advertised with an unsupported version.
    #[error("required Wayland global `{0}` does not support a usable version")]
    UnsupportedGlobalVersion(&'static str),
    /// The compositor did not advertise any outputs.
    #[error("the compositor did not advertise any outputs")]
    NoOutputs,
    /// The compositor closed the wallpaper surface.
    #[error("the compositor closed the wallpaper surface")]
    SurfaceClosed,
    /// The compositor configured the layer surface with an invalid size.
    #[error("the compositor configured an invalid wallpaper size {width}x{height}")]
    InvalidConfigureSize {
        /// Configured width.
        width: u32,
        /// Configured height.
        height: u32,
    },
    /// Buffer sizing overflowed integer bounds.
    #[error("the shared-memory buffer size overflowed")]
    BufferSizeOverflow,
    /// The rendered image size did not match the expected buffer size.
    #[error("rendered pixel data length {actual} did not match expected length {expected}")]
    BufferSizeMismatch {
        /// Expected byte length.
        expected: usize,
        /// Actual byte length.
        actual: usize,
    },
}
