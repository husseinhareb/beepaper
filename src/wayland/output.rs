//! Output discovery for the native Wayland apply path.

use wayland_client::{
    Dispatch, QueueHandle, WEnum,
    globals::GlobalList,
    protocol::wl_output::{self, WlOutput},
};

use crate::error::WaylandError;

/// Basic output sizing information needed by the wallpaper client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputMetrics {
    /// Current output width in physical pixels.
    pub width_px: Option<u32>,
    /// Current output height in physical pixels.
    pub height_px: Option<u32>,
    /// Output scale factor. Defaults to `1`.
    pub scale: i32,
}

impl Default for OutputMetrics {
    fn default() -> Self {
        Self {
            width_px: None,
            height_px: None,
            scale: 1,
        }
    }
}

impl OutputMetrics {
    /// Return the current logical output size, accounting for scale.
    pub fn logical_size(&self) -> Option<(u32, u32)> {
        let width = self.width_px?;
        let height = self.height_px?;
        let scale = self.scale.max(1) as u32;

        Some((width.max(scale) / scale, height.max(scale) / scale))
    }

    fn update_from_event(&mut self, event: &wl_output::Event) {
        match event {
            wl_output::Event::Mode {
                flags,
                width,
                height,
                ..
            } => {
                let WEnum::Value(flags) = flags else {
                    return;
                };

                if !flags.contains(wl_output::Mode::Current) {
                    return;
                }

                let Ok(width) = u32::try_from(*width) else {
                    return;
                };
                let Ok(height) = u32::try_from(*height) else {
                    return;
                };

                self.width_px = Some(width);
                self.height_px = Some(height);
            }
            wl_output::Event::Scale { factor } => {
                self.scale = (*factor).max(1);
            }
            _ => {}
        }
    }
}

/// The chosen output for the current wallpaper session.
#[derive(Debug, Clone)]
pub struct SelectedOutput {
    /// Registry name of the output global.
    pub registry_name: u32,
    /// Bound output proxy.
    pub output: WlOutput,
    /// Current known mode and scale details.
    pub metrics: OutputMetrics,
}

impl SelectedOutput {
    /// Apply an output event if it belongs to the selected output.
    pub fn handle_event(&mut self, output: &WlOutput, event: &wl_output::Event) {
        if output != &self.output {
            return;
        }

        self.metrics.update_from_event(event);
    }

    /// Return the known logical size of the selected output.
    pub fn logical_size(&self) -> Option<(u32, u32)> {
        self.metrics.logical_size()
    }

    /// Return the current scale factor, normalized to at least `1`.
    pub fn scale(&self) -> i32 {
        self.metrics.scale.max(1)
    }
}

/// Select the first advertised output.
pub fn select_first_output<D>(
    globals: &GlobalList,
    qh: &QueueHandle<D>,
) -> Result<SelectedOutput, WaylandError>
where
    D: Dispatch<WlOutput, ()> + 'static,
{
    let global = globals
        .contents()
        .clone_list()
        .into_iter()
        .find(|global| global.interface == "wl_output")
        .ok_or(WaylandError::NoOutputs)?;

    let output = globals
        .registry()
        .bind(global.name, global.version.min(4), qh, ());

    Ok(SelectedOutput {
        registry_name: global.name,
        output,
        metrics: OutputMetrics::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::OutputMetrics;

    #[test]
    fn logical_size_accounts_for_scale() {
        let metrics = OutputMetrics {
            width_px: Some(3840),
            height_px: Some(2160),
            scale: 2,
        };

        assert_eq!(metrics.logical_size(), Some((1920, 1080)));
    }

    #[test]
    fn logical_size_requires_mode_dimensions() {
        let metrics = OutputMetrics::default();

        assert_eq!(metrics.logical_size(), None);
    }
}
