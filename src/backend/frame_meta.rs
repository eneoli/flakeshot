use wayland_client::protocol::wl_shm::Format;

#[derive(Debug)]
pub struct FrameMeta {
    pub format: Option<Format>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}