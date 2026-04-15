//! Wayland global binding helpers.

use std::ops::RangeInclusive;

use wayland_client::{
    Dispatch, Proxy, QueueHandle,
    globals::{BindError, GlobalList},
    protocol::{wl_compositor::WlCompositor, wl_shm::WlShm},
};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1;

use crate::error::WaylandError;

/// Required globals for the native Wayland apply path.
#[derive(Debug, Clone)]
pub struct RequiredGlobals {
    /// The compositor singleton.
    pub compositor: WlCompositor,
    /// The shared-memory singleton.
    pub shm: WlShm,
    /// The wlr layer-shell global.
    pub layer_shell: ZwlrLayerShellV1,
}

impl RequiredGlobals {
    /// Bind the globals needed for the wallpaper MVP.
    pub fn bind<D>(globals: &GlobalList, qh: &QueueHandle<D>) -> Result<Self, WaylandError>
    where
        D: Dispatch<WlCompositor, ()>
            + Dispatch<WlShm, ()>
            + Dispatch<ZwlrLayerShellV1, ()>
            + 'static,
    {
        Ok(Self {
            compositor: bind_singleton(globals, qh, "wl_compositor", 1..=4)?,
            shm: bind_singleton(globals, qh, "wl_shm", 1..=1)?,
            layer_shell: bind_singleton(globals, qh, "zwlr_layer_shell_v1", 1..=4)?,
        })
    }
}

fn bind_singleton<I, D>(
    globals: &GlobalList,
    qh: &QueueHandle<D>,
    name: &'static str,
    version: RangeInclusive<u32>,
) -> Result<I, WaylandError>
where
    I: Proxy + 'static,
    D: Dispatch<I, ()> + 'static,
{
    match globals.bind(qh, version, ()) {
        Ok(proxy) => Ok(proxy),
        Err(BindError::NotPresent) => Err(WaylandError::MissingGlobal(name)),
        Err(BindError::UnsupportedVersion) => Err(WaylandError::UnsupportedGlobalVersion(name)),
    }
}
