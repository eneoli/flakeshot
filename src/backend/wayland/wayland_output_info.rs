use crate::backend::wayland::wayland_geometry::WaylandGeometry;
use crate::backend::wayland::wayland_output_mode::WaylandOutputMode;
use crate::backend::{MonitorInfo, OutputInfo, Pixel};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::Proxy;

/// This represents a wayland output (aka. a monitor).
#[derive(Clone, Debug)]
pub struct WaylandOutputInfo {
    pub output: WlOutput,
    pub name: String,
    pub description: String,
    pub scale: i32,
    pub geometry: WaylandGeometry,
    pub mode: WaylandOutputMode,
}

impl WaylandOutputInfo {
    pub fn from_wl_output(wl_ouput: WlOutput) -> WaylandOutputInfo {
        Self {
            output: wl_ouput,
            scale: 0,
            description: String::new(),
            name: String::new(),
            geometry: WaylandGeometry::default(),
            mode: WaylandOutputMode::default(),
        }
    }
}

impl From<&WaylandOutputInfo> for OutputInfo {
    fn from(value: &WaylandOutputInfo) -> Self {
        OutputInfo {
            id: value.output.id().protocol_id(),
            width: value.mode.width as Pixel,
            height: value.mode.height as Pixel,
            x: value.geometry.x as i16,
            y: value.geometry.y as i16,
            monitor_info: MonitorInfo::Wayland {
                name: value.name.clone(),
                description: value.description.clone(),
            },
        }
    }
}
