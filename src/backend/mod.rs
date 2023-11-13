//! Contains the different backends to get the screenshot from.

pub mod wayland;
pub mod x11;

/// A general backend error enum.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Represents that an error occured while trying to get a screenshot on X11.
    #[error(transparent)]
    X11(#[from] x11::Error),
}

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

/// The main function of this module.
///
/// This function returns an rgb-image for each screen (or "monitor" in other
/// words).
pub async fn create_screenshots() -> anyhow::Result<Vec<(OutputInfo, image::DynamicImage)>> {
    todo!()
}
