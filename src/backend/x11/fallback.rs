use image::{DynamicImage, RgbImage, RgbaImage};
use x11rb::{
    connection::Connection,
    protocol::xproto::{ImageFormat, ImageOrder, Screen},
    rust_connection::RustConnection,
};

use super::ScreenshotCreator;

#[derive(Debug)]
pub struct Fallback;

impl ScreenshotCreator for Fallback {
    fn get_image(
        &self,
        conn: &RustConnection,
        screen: &Screen,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<DynamicImage, super::Error> {
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
