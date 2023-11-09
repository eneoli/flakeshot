use wayland_client::protocol::wl_output;

#[derive(Default, Debug)]
pub struct OutputMode {
    pub height: i32,
    pub width: i32,
    pub refresh: i32,
    // flags
}

impl OutputMode {
    pub fn from_wayland_event(event: wl_output::Event) -> Option<Self> {
        if let wl_output::Event::Mode { height, width, refresh, .. } = event {
            Some(
                Self {
                    height: height,
                    width: width,
                    refresh: refresh,
                }
            )
        } else {
            None
        }
    }
}