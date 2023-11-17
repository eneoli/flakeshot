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

/// An alias type for better code readability.
pub type Pixel = u16;

/// This enum contains specifique values depending on which backend.
#[derive(Debug)]
pub enum MonitorInfo {
    /// Some additional values in the X11 context.
    X11 {
        ///
        name: u32,
    },

    /// Some additional values in the wayland context.
    Wayland { name: String, description: String },
}

#[derive(Debug)]
pub struct OutputInfo {
    pub width: Pixel,
    pub height: Pixel,
    pub x: i16,
    pub y: i16,

    pub id: u32,
    pub monitor_info: MonitorInfo,
}

/// The main function of this module.
///
/// # General description
/// This function returns an image for each screen (or "monitor" in other
/// words).
///
/// # Return value
/// A tuple where the first value contains additional information about the image which is the second
/// image.
pub async fn create_screenshots() -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    let xorg_is_running = { x11rb::connect(None).is_ok() };

    if xorg_is_running {
        x11::get_images().map_err(Error::from)
    } else {
        todo!()
    }
}
