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

#[derive(Debug)]
pub enum AdditionalInformation {
    X11 { name: u32 },
    Wayland { name: String, description: String },
}

#[derive(Debug)]
pub struct OutputInfo {
    pub width: Pixel,
    pub height: Pixel,
    pub x: i16,
    pub y: i16,

    pub id: u32,
    pub info: AdditionalInformation,
}

/// The main function of this module.
///
/// This function returns an rgb-image for each screen (or "monitor" in other
/// words).
pub async fn create_screenshots() -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    let xorg_is_running = std::env::var("DISPLAY").is_ok();

    if xorg_is_running {
        x11::get_images().map_err(Error::from)
    } else {
        todo!()
    }
}
