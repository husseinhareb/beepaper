//! Central Wayland app state and event-loop orchestration.

use std::path::Path;

use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, delegate_noop,
    globals::{GlobalList, GlobalListContents, registry_queue_init},
    protocol::{
        wl_buffer::WlBuffer, wl_compositor::WlCompositor, wl_output::WlOutput,
        wl_registry::WlRegistry, wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_surface::WlSurface,
    },
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{Event as LayerSurfaceEvent, ZwlrLayerSurfaceV1},
};

use super::{
    ApplyOptions,
    buffer::ShmBuffer,
    globals::RequiredGlobals,
    layer::LayerSurfaceState,
    output::{SelectedOutput, select_first_output},
    render::render_image,
};
use crate::error::WaylandError;

/// Event-dispatch state for the wallpaper client.
#[derive(Debug, Default)]
pub struct WaylandState {
    layer: Option<LayerSurfaceState>,
    output: Option<SelectedOutput>,
}

impl WaylandState {
    fn set_layer(&mut self, layer: LayerSurfaceState) {
        self.layer = Some(layer);
    }

    fn layer(&self) -> Option<&LayerSurfaceState> {
        self.layer.as_ref()
    }

    fn layer_mut(&mut self) -> Option<&mut LayerSurfaceState> {
        self.layer.as_mut()
    }

    fn set_output(&mut self, output: SelectedOutput) {
        self.output = Some(output);
    }

    fn output(&self) -> Option<&SelectedOutput> {
        self.output.as_ref()
    }

    fn output_mut(&mut self) -> Option<&mut SelectedOutput> {
        self.output.as_mut()
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut Self,
        _registry: &WlRegistry,
        _event: wayland_client::protocol::wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        layer_surface: &ZwlrLayerSurfaceV1,
        event: LayerSurfaceEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let Some(layer) = state.layer_mut() {
            layer.handle_event(layer_surface, event);
        }
    }
}

impl Dispatch<WlOutput, ()> for WaylandState {
    fn event(
        state: &mut Self,
        output: &WlOutput,
        event: wayland_client::protocol::wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let Some(selected_output) = state.output_mut() {
            selected_output.handle_event(output, &event);
        }
    }
}

delegate_noop!(WaylandState: ignore WlShm);
delegate_noop!(WaylandState: ignore WlShmPool);
delegate_noop!(WaylandState: ignore WlBuffer);
delegate_noop!(WaylandState: ignore WlCompositor);
delegate_noop!(WaylandState: ignore WlSurface);
delegate_noop!(WaylandState: ignore ZwlrLayerShellV1);

/// A prepared native wallpaper session that has already committed its first frame.
#[derive(Debug)]
pub struct PreparedWallpaper {
    app: WaylandApp,
    state: WaylandState,
    _buffer: ShmBuffer,
}

impl PreparedWallpaper {
    /// Keep the wallpaper surface alive until the compositor closes it or the process exits.
    pub fn run(mut self) -> Result<(), WaylandError> {
        self.app.run_until_closed(&mut self.state)
    }
}

/// Top-level Wayland app wrapper.
#[derive(Debug)]
pub struct WaylandApp {
    _connection: Connection,
    globals: GlobalList,
    event_queue: EventQueue<WaylandState>,
    qh: QueueHandle<WaylandState>,
}

impl WaylandApp {
    fn connect() -> Result<Self, WaylandError> {
        let connection = Connection::connect_to_env()?;
        let (globals, event_queue) = registry_queue_init::<WaylandState>(&connection)?;
        let qh = event_queue.handle();

        Ok(Self {
            _connection: connection,
            globals,
            event_queue,
            qh,
        })
    }

    /// Prepare a wallpaper surface and return only after the initial frame has been committed.
    pub fn prepare(path: &Path, options: &ApplyOptions) -> Result<PreparedWallpaper, WaylandError> {
        let mut app = Self::connect()?;
        let required = RequiredGlobals::bind(&app.globals, &app.qh)?;
        let output = select_first_output(&app.globals, &app.qh)?;

        let mut state = WaylandState::default();
        state.set_output(output.clone());
        state.set_layer(LayerSurfaceState::create(
            &required,
            &output.output,
            &app.qh,
        ));

        let surface_size = app.wait_for_initial_configure(&mut state)?;
        let rendered = render_image(
            path,
            options.mode,
            surface_size.buffer_width,
            surface_size.buffer_height,
        )?;
        let buffer = ShmBuffer::new(
            &required.shm,
            &app.qh,
            rendered.width,
            rendered.height,
            &rendered.pixels,
        )?;

        if let Some(layer) = state.layer_mut() {
            layer.attach_and_commit(
                &buffer.buffer,
                surface_size.logical_width as i32,
                surface_size.logical_height as i32,
                surface_size.scale,
            );
        }

        app.event_queue.roundtrip(&mut state)?;
        if state.layer().is_some_and(|layer| layer.is_closed()) {
            return Err(WaylandError::SurfaceClosed);
        }

        Ok(PreparedWallpaper {
            app,
            state,
            _buffer: buffer,
        })
    }

    fn wait_for_initial_configure(
        &mut self,
        state: &mut WaylandState,
    ) -> Result<SurfaceSize, WaylandError> {
        loop {
            if let Some(layer) = state.layer() {
                if layer.is_closed() {
                    return Err(WaylandError::SurfaceClosed);
                }

                if let Some(configure) = layer.configure() {
                    if let Some(surface_size) = Self::resolve_surface_size(state, configure)? {
                        return Ok(surface_size);
                    }
                }
            }

            self.event_queue.blocking_dispatch(state)?;
        }
    }

    fn run_until_closed(&mut self, state: &mut WaylandState) -> Result<(), WaylandError> {
        loop {
            if state.layer().is_some_and(|layer| layer.is_closed()) {
                return Ok(());
            }

            self.event_queue.blocking_dispatch(state)?;
        }
    }

    fn resolve_surface_size(
        state: &WaylandState,
        configure: super::layer::LayerConfigure,
    ) -> Result<Option<SurfaceSize>, WaylandError> {
        let output = state.output().ok_or(WaylandError::NoOutputs)?;
        let output_size = output.logical_size();

        let logical_width = if configure.width == 0 {
            let Some((width, _)) = output_size else {
                return Ok(None);
            };
            width
        } else {
            configure.width
        };

        let logical_height = if configure.height == 0 {
            let Some((_, height)) = output_size else {
                return Ok(None);
            };
            height
        } else {
            configure.height
        };

        if logical_width == 0 || logical_height == 0 {
            return Err(WaylandError::InvalidConfigureSize {
                width: logical_width,
                height: logical_height,
            });
        }

        let scale = output.scale();
        let scale_u32 = scale as u32;
        let buffer_width = logical_width
            .checked_mul(scale_u32)
            .ok_or(WaylandError::BufferSizeOverflow)?;
        let buffer_height = logical_height
            .checked_mul(scale_u32)
            .ok_or(WaylandError::BufferSizeOverflow)?;

        Ok(Some(SurfaceSize {
            logical_width,
            logical_height,
            buffer_width,
            buffer_height,
            scale,
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SurfaceSize {
    logical_width: u32,
    logical_height: u32,
    buffer_width: u32,
    buffer_height: u32,
    scale: i32,
}
