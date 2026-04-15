//! Layer-shell surface lifecycle management.

use wayland_client::{
    Dispatch, QueueHandle,
    protocol::{wl_buffer::WlBuffer, wl_output::WlOutput, wl_surface::WlSurface},
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer,
    zwlr_layer_surface_v1::{
        Anchor, Event as LayerSurfaceEvent, KeyboardInteractivity, ZwlrLayerSurfaceV1,
    },
};

use super::globals::RequiredGlobals;

/// The compositor-provided size for the wallpaper surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerConfigure {
    /// Configure serial for `ack_configure`.
    pub serial: u32,
    /// Width assigned by the compositor.
    pub width: u32,
    /// Height assigned by the compositor.
    pub height: u32,
}

/// State for the wallpaper layer surface.
#[derive(Debug)]
pub struct LayerSurfaceState {
    /// Backing wl_surface.
    pub surface: WlSurface,
    /// Layer-shell wrapper for the surface.
    pub layer_surface: ZwlrLayerSurfaceV1,
    configured: Option<LayerConfigure>,
    mapped: bool,
    closed: bool,
}

impl LayerSurfaceState {
    /// Create a background layer surface and request its initial configure.
    pub fn create<D>(globals: &RequiredGlobals, output: &WlOutput, qh: &QueueHandle<D>) -> Self
    where
        D: Dispatch<WlSurface, ()> + Dispatch<ZwlrLayerSurfaceV1, ()> + 'static,
    {
        let surface = globals.compositor.create_surface(qh, ());
        let layer_surface = globals.layer_shell.get_layer_surface(
            &surface,
            Some(output),
            Layer::Background,
            "beepaper".to_string(),
            qh,
            (),
        );

        layer_surface.set_size(0, 0);
        layer_surface.set_anchor(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
        layer_surface.set_margin(0, 0, 0, 0);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.set_exclusive_zone(-1);
        surface.commit();

        Self {
            surface,
            layer_surface,
            configured: None,
            mapped: false,
            closed: false,
        }
    }

    /// Process a layer-surface event.
    pub fn handle_event(&mut self, layer_surface: &ZwlrLayerSurfaceV1, event: LayerSurfaceEvent) {
        match event {
            LayerSurfaceEvent::Configure {
                serial,
                width,
                height,
            } => {
                layer_surface.ack_configure(serial);
                self.configured = Some(LayerConfigure {
                    serial,
                    width,
                    height,
                });
            }
            LayerSurfaceEvent::Closed => {
                self.closed = true;
            }
            _ => {}
        }
    }

    /// Return the latest configure received from the compositor.
    pub fn configure(&self) -> Option<LayerConfigure> {
        self.configured
    }

    /// Return whether the compositor has closed the layer surface.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Return whether a buffer has been attached and committed.
    pub fn is_mapped(&self) -> bool {
        self.mapped
    }

    /// Attach the rendered wallpaper buffer and commit the surface.
    pub fn attach_and_commit(&mut self, buffer: &WlBuffer, width: i32, height: i32, scale: i32) {
        self.surface.set_buffer_scale(scale.max(1));
        self.surface.attach(Some(buffer), 0, 0);
        self.surface.damage(0, 0, width, height);
        self.surface.commit();
        self.mapped = true;
    }
}
