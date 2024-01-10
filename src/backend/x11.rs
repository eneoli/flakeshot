//! Backend implementation for X11.
use image::{DynamicImage, RgbImage, RgbaImage};
use x11rb::{
    connection::Connection,
    protocol::xproto::{ImageFormat, ImageOrder, Screen},
    rust_connection::RustConnection,
};

use super::{MonitorInfo, OutputInfo};

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

    #[error(transparent)]
    StringUtf8(#[from] std::string::FromUtf8Error),
}

/// The main function of this module.
///
/// This function collects, from each screen (a.k.a your monitors) a screenshot
/// and returns it.
///
/// # Example
/// ```no_test
/// use flakeshot::backend::x11::get_images;
/// use std::fs::File;
/// use image::ImageOutputFormat;
///
/// fn main() {
///     let mut file = File::create("./targets/example_screenshot.png").unwrap();
///     let images = get_images().unwrap();
///
///     // we will only use the first screenshot for this example
///     let first_screen = images.first().unwrap();
///     let image = &first_screen.1;
///
///     image.write_to(&mut file, ImageOutputFormat::Png).unwrap();
/// }
/// ```
pub fn create_screenshots() -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    use x11rb::protocol::randr::ConnectionExt;

    let (conn, _) = x11rb::connect(None)?;
    let setup = conn.setup();

    let mut images = Vec::with_capacity(setup.roots.len());

    for screen in &setup.roots {
        let image = get_image(&conn, screen)?;
        let monitors = conn.randr_get_monitors(screen.root, true)?.reply()?;

        for monitor in &monitors.monitors {
            assert!(
                monitor.outputs.len() == 1,
                "We currently support only one output for each monitor. Please create an issue if you encounter this assert."
            );

            let monitor_info = {
                let screen_resources = conn
                    .randr_get_screen_resources_current(screen.root)?
                    .reply()?;

                let output_name = {
                    let output_info = conn
                        .randr_get_output_info(
                            monitor.outputs[0],
                            screen_resources.config_timestamp,
                        )?
                        .reply()?;

                    String::from_utf8(output_info.name)?
                };

                MonitorInfo::X11 { output_name }
            };

            let output_info = OutputInfo {
                id: screen.root,

                width: monitor.width,
                height: monitor.height,
                x: monitor.x,
                y: monitor.y,

                monitor_info,
            };

            images.push((output_info, image.clone()));
        }
    }

    Ok(images)
}

fn get_image(conn: &RustConnection, screen: &Screen) -> Result<DynamicImage, Error> {
    use x11rb::protocol::xproto::ConnectionExt;
    const ALL_BITS: u32 = u32::MAX;

    let setup = &conn.setup();
    let width_u16 = screen.width_in_pixels;
    let height_u16 = screen.height_in_pixels;

    let width_u32 = u32::from(width_u16);
    let height_u32 = u32::from(height_u16);

    let (image_bytes, pixmap_format) = {
        let image_reply = conn
            .get_image(
                ImageFormat::Z_PIXMAP,
                screen.root,
                0,
                0,
                width_u16,
                height_u16,
                ALL_BITS,
            )
            .map_err(Error::from)?
            .reply()
            .map_err(Error::from)?;

        let pixmap_format = setup
            .pixmap_formats
            .iter()
            .find(|format| format.depth == image_reply.depth)
            .unwrap();

        (image_reply.data, pixmap_format)
    };

    let bit_order = setup.bitmap_format_bit_order;
    let image = match pixmap_format.bits_per_pixel {
        24 => get_rgb_image(width_u32, height_u32, image_bytes, bit_order),
        32 => get_rgba_image(width_u32, height_u32, image_bytes, bit_order),
        _ => unimplemented!(
            "We don't support {}-bit RGB values",
            pixmap_format.bits_per_pixel
        ),
    };

    Ok(image)
}

fn get_rgb_image(
    width: u32,
    height: u32,
    image_bytes: Vec<u8>,
    bit_order: ImageOrder,
) -> DynamicImage {
    let mut rgb_image = RgbImage::from_vec(width, height, image_bytes).unwrap();
    if bit_order == ImageOrder::LSB_FIRST {
        for rgb in rgb_image.pixels_mut() {
            rgb.0.reverse();
        }
    }
    DynamicImage::ImageRgb8(rgb_image)
}

fn get_rgba_image(
    width: u32,
    height: u32,
    image_bytes: Vec<u8>,
    bit_order: ImageOrder,
) -> DynamicImage {
    let mut rgba_image = RgbaImage::from_vec(width, height, image_bytes).unwrap();

    if bit_order == ImageOrder::LSB_FIRST {
        for rgba in rgba_image.pixels_mut() {
            rgba.0[0..3].reverse();
        }
    }
    DynamicImage::ImageRgba8(rgba_image)
}
