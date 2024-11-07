//! Backend implementation for X11.
use image::{DynamicImage, RgbImage, RgbaImage};
use tracing::info;
use x11rb::{
    connection::Connection,
    protocol::{
        randr::ConnectionExt,
        xproto::{ImageFormat, ImageOrder, Screen},
    },
    rust_connection::RustConnection,
};

use super::{MonitorInfo, OutputInfo};

/// Arguments:
/// - conn
/// - screen
/// - x
/// - y
/// - width
/// - height
type FnCreateScreenshot =
    dyn Fn(&RustConnection, &Screen, i16, i16, u16, u16) -> Result<DynamicImage, Error>;

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

    #[error("Couldn't create screenshots with portal: {0}")]
    Portal(#[from] super::portal::Error),
}

/// The main function of this module.
/// Tries to retrieve a screenshot of each monitor by attempting different ways (for example by using portals.)
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
    match try_with_portal() {
        Ok(screenshots) => return Ok(screenshots),
        Err(e) => info!("X11: {}", e),
    }

    inner_create_screenshots(&manual_create_screenshot)
}

/// A generalized function which iterates through all screens and creates a screenshot of it.
///
/// # Arguments
/// - `create_screenshot_fn`: This function will be called for each screen and it should return the screenshot with the given data of
///                           the screen
fn inner_create_screenshots(
    create_screenshot_fn: &FnCreateScreenshot,
) -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    let (conn, _) = x11rb::connect(None)?;
    let setup = conn.setup();

    let mut images = Vec::with_capacity(setup.roots.len());

    for screen in &setup.roots {
        let monitors = conn.randr_get_monitors(screen.root, true)?.reply()?;

        for monitor in &monitors.monitors {
            assert!(
                monitor.outputs.len() == 1,
                "We currently support only one output for each monitor. Please create an issue if you encounter this assert."
            );

            let image = create_screenshot_fn(
                &conn,
                screen,
                monitor.x,
                monitor.y,
                monitor.width,
                monitor.height,
            )?;
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

                MonitorInfo::X11 { name: output_name }
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

/// This function communicates directly with the xorg-server to create the screenshot (that's why we call it "manually")
fn manual_create_screenshot(
    conn: &RustConnection,
    screen: &Screen,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
) -> Result<DynamicImage, Error> {
    use x11rb::protocol::xproto::ConnectionExt;
    const ALL_BITS: u32 = u32::MAX;

    let setup = &conn.setup();
    let width_u32 = u32::from(width);
    let height_u32 = u32::from(height);

    let (image_bytes, pixmap_format) = {
        let image_reply = conn
            .get_image(
                ImageFormat::Z_PIXMAP,
                screen.root,
                x,
                y,
                width,
                height,
                ALL_BITS,
            )?
            .reply()?;

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

/// This function attempts to create the screenshot by using portals
fn try_with_portal() -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    let screenshot = crate::backend::portal::create_screenshot()?;

    #[rustfmt::skip]
    let crop = move |_conn: &RustConnection, _screen: &Screen, x: i16, y: i16, width: u16, height: u16| -> Result<DynamicImage, Error> {
        Ok(screenshot.crop_imm(x as u32, y as u32, width.into(), height.into()))
    };

    inner_create_screenshots(&crop)
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
