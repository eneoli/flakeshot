//! Contains the different backends to get the screenshot from.

use std::{
    io::{Cursor, Write},
    process::Stdio,
};

use image::{GenericImageView, ImageOutputFormat};
use relm4::factory::DynamicIndex;

pub mod wayland;
pub mod x11;

/// Represents an error which occured in one of the backends
/// while trying to get the screenshot(s) of each output.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Represents that an error occured while trying to get a screenshot on X11.
    #[error(transparent)]
    X11(#[from] x11::Error),

    /// Represents that an error occured while trying to get a screenshot on Wayland.
    #[error(transparent)]
    Wayland(#[from] wayland::wayland_error::WaylandError),
}

/// An alias type for better code readability.
pub type Pixel = u16;

/// Contains additional values depending on the backend.
#[derive(Debug, Clone)]
pub enum MonitorInfo {
    /// Some additional values in the X11 context.
    X11 { name: String },

    /// Some additional values in the wayland context.
    Wayland { name: String, description: String },
}

/// Some general information about an output.
#[derive(Debug, Clone)]
pub struct OutputInfo {
    /// The width of the output.
    pub width: Pixel,

    /// The height of the output.
    pub height: Pixel,

    /// The x-value of the top-left corner of the output.
    pub x: i16,

    /// The y-value of the top-left corner of the output.
    pub y: i16,

    /// The id of the output.
    pub id: u32,

    /// Some additional information about the monitor
    pub monitor_info: MonitorInfo,
}

/// Checks if system is using Wayland
pub fn is_wayland() -> bool {
    wayland_client::Connection::connect_to_env().is_ok()
}

/// The main function of this module.
///
/// # General description
/// This function returns an image for each screen (or "monitor" in other
/// words).
///
/// # Return value
/// A tuple where the first value contains some general information about the output and is
/// mapped to the given image in the second value of the tuple.
pub fn create_screenshots() -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    if is_wayland() {
        wayland::create_screenshots().map_err(Error::from)
    } else {
        x11::create_screenshots().map_err(Error::from)
    }
}

pub fn save_to_clipboard(img: image::DynamicImage) {
    let mut child = if is_wayland() {
        std::process::Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
            .unwrap()
    } else {
        std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-target", "image/png"])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap()
    };

    let mut image_bytes: Vec<u8> = {
        let dim = img.dimensions();
        Vec::with_capacity((dim.0 * dim.1) as usize)
    };

    img.write_to(&mut Cursor::new(&mut image_bytes), ImageOutputFormat::Png)
        .unwrap();

    {
        let child_stdin = child.stdin.as_mut().unwrap();
        child_stdin.write_all(&image_bytes).unwrap();
    }
}
