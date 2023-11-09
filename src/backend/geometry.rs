use wayland_client::protocol::wl_output;

#[derive(Default, Debug)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub physical_width: i32,
    pub physical_height: i32,
    pub model: String,
    pub make: String,
    // transform
    // subpixel
}

impl Geometry {
    pub fn from_wayland_geometry(event: wl_output::Event) -> Option<Geometry> {
        if let wl_output::Event::Geometry {
            x,
            y,
            physical_width,
            physical_height,
            model,
            make,
            ..
        } = event {
            Some(
                Geometry {
                    x,
                    y,
                    physical_width,
                    physical_height,
                    model,
                    make,
                }
            )
        } else {
            None
        }
    }
}