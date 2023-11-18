use wayland_client::protocol::wl_shm::Format;

/// This represents metadata of a frame made by `zwlr_screencopy_manager_v1`.
/// A frame is what we call a screenshot of a single wayland output (aka a monitor).
#[derive(Clone, Debug)]
pub struct WaylandFrameMeta {
    pub format: Option<Format>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}
