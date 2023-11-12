//! Backend implementation for X11.
//!
//! # Credits
//! This module is inspired by the [`screenshots`] crate.
//!
//! [`screenshots`]: https://crates.io/crates/screenshots
use image::{DynamicImage, RgbImage, RgbaImage};
use x11rb::{
    connection::Connection,
    protocol::xproto::{ConnectionExt, ImageFormat, ImageOrder},
};

const ALL_BITS_FROM_PLANE: u32 = u32::MAX;

/// A general enum with possible errors as values which can occur while
/// operating with the xorg-server.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't connect to the xorg server: {0}")]
    ConnectError(#[from] x11rb::errors::ConnectError),

    #[error("The connection broke with the xorg server: {0}")]
    ConnectionError(#[from] x11rb::errors::ConnectionError),

    #[error("Couldn't request an image from the xorg-server: {0}")]
    ReplyError(#[from] x11rb::errors::ReplyError),
}

/// The main function of this module.
///
/// This function collects, from each screen (a.k.a your monitors) a screenshot
/// and returns it.
///
/// # Example
/// ```no_run
/// use flakeshot::backend::x11::get_images;
/// use std::fs::File;
/// use image::ImageOutputFormat;
///
/// fn main() {
///     let mut file = File::create("./targets/example_screenshot.png").unwrap();
///     let images = get_images().unwrap();
///
///     // we will only use the first screenshot for this example
///     let first_image = images.first().unwrap();
///
///     first_image.write_to(&mut file, ImageOutputFormat::Png).unwrap();
/// }
/// ```
pub fn get_images() -> Result<Vec<image::DynamicImage>, Error> {
    let (conn, _) = x11rb::connect(None)?;
    let setup = conn.setup();

    let mut images = Vec::with_capacity(setup.roots.len());

    for screen in &setup.roots {
        let width_u16 = screen.width_in_pixels;
        let height_u16 = screen.height_in_pixels;

        let width_u32 = u32::from(width_u16);
        let height_u32 = u32::from(height_u16);

        let (image_bytes, pixmap_format) = {
            let cookie = conn
                .get_image(
                    ImageFormat::Z_PIXMAP,
                    screen.root,
                    0,
                    0,
                    width_u16,
                    height_u16,
                    ALL_BITS_FROM_PLANE,
                )
                .map_err(Error::from)?;

            let image_reply = cookie.reply().map_err(Error::from)?;
            let pixmap_format = setup
                .pixmap_formats
                .iter()
                .find(|format| format.depth == image_reply.depth)
                .unwrap();

            (image_reply.data, pixmap_format)
        };

        let image = match pixmap_format.bits_per_pixel {
            24 => {
                let mut rgb_image = RgbImage::from_vec(width_u32, height_u32, image_bytes).unwrap();
                if setup.bitmap_format_bit_order == ImageOrder::LSB_FIRST {
                    for rgb in rgb_image.pixels_mut() {
                        rgb.0.reverse();
                    }
                }
                DynamicImage::ImageRgb8(rgb_image)
            }
            32 => {
                let mut rgba_image =
                    RgbaImage::from_vec(width_u32, height_u32, image_bytes).unwrap();

                if setup.bitmap_format_bit_order == ImageOrder::LSB_FIRST {
                    for rgba in rgba_image.pixels_mut() {
                        rgba.0[0..3].reverse();
                    }
                }
                DynamicImage::ImageRgba8(rgba_image)
            }
            _ => unimplemented!(
                "We don't support {}-bit RGB values",
                pixmap_format.bits_per_pixel
            ),
        };

        images.push(image);
    }

    Ok(images)
}
