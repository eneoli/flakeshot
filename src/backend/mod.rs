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

/// The main function of this module.
///
/// This function returns an rgb-image for each screen (or "monitor" in other
/// words).
pub fn get_images() -> Result<Vec<image::DynamicImage>, Error> {
    let xorg_server_is_running = std::env::var("DISPLAY").is_ok();

    if xorg_server_is_running {
        x11::get_images().map_err(Error::from)
    } else {
        todo!()
    }
}
