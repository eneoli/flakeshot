use image::{Rgb, RgbImage};
use x11rb::{
    connection::Connection,
    protocol::xproto::{ConnectionExt, ImageFormat, ImageOrder},
};

const ALL_BITS_FROM_PLANE: u32 = u32::MAX;
const U8_MAX: f32 = u8::MAX as f32;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't connect to the xorg server: {0}")]
    ConnectError(#[from] x11rb::errors::ConnectError),

    #[error("The connection broke with the xorg server: {0}")]
    ConnectionError(#[from] x11rb::errors::ConnectionError),

    #[error("Couldn't request an image from the xorg-server: {0}")]
    ReplyError(#[from] x11rb::errors::ReplyError),
}

pub fn get_images() -> Result<Vec<image::RgbImage>, Error> {
    let (conn, _) = x11rb::connect(None)?;
    let setup = conn.setup();

    let mut images = Vec::with_capacity(setup.roots.len());

    for screen in &setup.roots {
        let width = screen.width_in_pixels as u32;
        let height = screen.width_in_pixels as u32;

        let (image_bytes, pixmap_format) = {
            let cookie = conn
                .get_image(
                    ImageFormat::Z_PIXMAP,
                    screen.root,
                    0,
                    0,
                    width as u16,
                    height as u16,
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

        let bytes_per_pixel = pixmap_format.bits_per_pixel as usize / 8;

        let get_rgb = match bytes_per_pixel {
            1 => get_8bit_rgb,
            2 => get_16bit_rgb,
            3 => get_32bit_rgb,
            _ => unimplemented!(
                "We didn't implement the rgb-extraction of a pixel consisting of {} bytes.",
                bytes_per_pixel
            ),
        };

        let mut image: Vec<u8> = Vec::with_capacity((width * height) as usize * bytes_per_pixel);

        for chunk in image_bytes.chunks_exact(bytes_per_pixel) {
            let mut rgb = get_rgb(chunk, setup.bitmap_format_bit_order);
            image.extend_from_slice(&mut rgb.0);
        }

        let image = RgbImage::from_vec(width, height, image).unwrap();
        images.push(image);
    }

    Ok(images)
}

fn get_8bit_rgb(chunk: &[u8], bit_order: ImageOrder) -> Rgb<u8> {
    const MAX_R_AND_B_VALUE: u8 = 0b11u8;
    const MAX_R_AND_B_VALUE_F32: f32 = MAX_R_AND_B_VALUE as f32;

    const MAX_G_VALUE: u8 = 0b111u8;
    const MAX_G_VALUE_F32: f32 = MAX_G_VALUE as f32;

    let lsb_pixel = get_8bit_pixel(chunk, bit_order);

    let r = {
        let red_bits = lsb_pixel >> 6;
        let red_ratio = (red_bits as f32) / MAX_R_AND_B_VALUE_F32;

        red_ratio * U8_MAX
    };

    let g = {
        let green_bits = (lsb_pixel >> 2) & MAX_G_VALUE;
        let green_ratio = (green_bits as f32) / MAX_G_VALUE_F32;

        green_ratio * U8_MAX
    };

    let b = {
        let blue_bits = lsb_pixel & MAX_R_AND_B_VALUE;
        let blue_ratio = (blue_bits as f32) / MAX_R_AND_B_VALUE_F32;

        blue_ratio * U8_MAX
    };

    Rgb([r as u8, g as u8, b as u8])
}

fn get_8bit_pixel(chunk: &[u8], bit_order: ImageOrder) -> u8 {
    if bit_order == ImageOrder::LSB_FIRST {
        chunk[0]
    } else {
        let head = chunk[0] & 0b111 << 4;
        let tail = chunk[0] >> 4;

        head | tail
    }
}

fn get_16bit_rgb(chunk: &[u8], bit_order: ImageOrder) -> Rgb<u8> {
    const MAX_R_AND_B_VALUE: u16 = 0b1_1111;
    const MAX_R_AND_B_VALUE_F32: f32 = MAX_R_AND_B_VALUE as f32;

    const MAX_G_VALUE: u16 = 0b11_1111;
    const MAX_G_VALUE_F32: f32 = MAX_G_VALUE as f32;

    let pixel = get_16bit_pixel(chunk, bit_order);

    let r = {
        let red_bits = pixel >> 11;
        let red_ratio = (red_bits as f32) / MAX_R_AND_B_VALUE_F32;

        red_ratio * U8_MAX
    };
    let g = {
        let green_bits = (pixel >> 5) & MAX_G_VALUE;
        let green_ratio = (green_bits as f32) / MAX_G_VALUE_F32;

        green_ratio * U8_MAX
    };
    let b = {
        let blue_bits = pixel & MAX_R_AND_B_VALUE;
        let blue_ratio = (blue_bits as f32) / MAX_R_AND_B_VALUE_F32;

        blue_ratio * U8_MAX
    };

    Rgb([r as u8, g as u8, b as u8])
}

fn get_16bit_pixel(chunk: &[u8], bit_order: ImageOrder) -> u16 {
    let u16_bytes = [chunk[0], chunk[1]];

    if bit_order == ImageOrder::LSB_FIRST {
        u16::from_be_bytes(u16_bytes)
    } else {
        u16::from_le_bytes(u16_bytes)
    }
}

fn get_32bit_rgb(chunk: &[u8], bit_order: ImageOrder) -> Rgb<u8> {
    let mut rgb = [chunk[0], chunk[1], chunk[2]];

    if bit_order == ImageOrder::LSB_FIRST {
        rgb.reverse();
        Rgb(rgb)
    } else {
        Rgb(rgb)
    }
}
