use wayland_client::protocol::wl_output::{Subpixel, Transform};

/// This represents physical metadata of a wayland output (aka a monitor).
#[derive(Default, Clone, Debug)]
pub struct WaylandGeometry {
    pub x: i32,
    pub y: i32,
    pub physical_width: i32,
    pub physical_height: i32,
    pub model: String,
    pub make: String,
    pub subpixel: Option<Subpixel>,
    pub transform: Option<Transform>,
}