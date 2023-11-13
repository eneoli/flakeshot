pub mod wayland;
pub mod xorg;

pub type Pixel = u16;

pub enum OutputIdentifier {
    X11(u32),
    Wayland {
        id: u32,
        name: String,
        description: String,
    },
}

pub struct OutputInfo {
    pub identifier: OutputIdentifier,
    pub width: Pixel,
    pub height: Pixel,
    pub x: i16,
    pub y: i16,
}

pub async fn create_screenshots() -> anyhow::Result<Vec<(OutputInfo, image::DynamicImage)>> {
    todo!()
}
