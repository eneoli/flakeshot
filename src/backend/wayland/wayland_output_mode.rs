use wayland_client::protocol::wl_output::Mode;

#[derive(Default, Clone, Debug)]
pub struct WaylandOutputMode {
    pub height: i32,
    pub width: i32,
    pub refresh: i32,
    pub flags: Option<Mode>,
}