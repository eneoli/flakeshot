use cairo::{Context, Format, ImageSurface};
use image::{DynamicImage, RgbaImage};

#[derive(Debug)]
pub struct Canvas {
    surface: ImageSurface,
}

impl Canvas {
    pub fn new(width: i32, height: i32) -> anyhow::Result<Self> {
        Ok(Canvas {
            surface: ImageSurface::create(Format::ARgb32, width, height)?,
        })
    }

    pub fn stamp_image(&self, x: f64, y: f64, image: &DynamicImage) -> anyhow::Result<()> {
        let ctx = Context::new(&self.surface)?;

        let mut image_bytes = Vec::from(image.as_bytes());
        Self::invert_rgba_vec(&mut image_bytes);

        let image_surface = ImageSurface::create_for_data(
            image_bytes,
            cairo::Format::ARgb32,
            image.width() as i32,
            image.height() as i32,
            Format::stride_for_width(Format::ARgb32, image.width() as u32)?,
        )?;

        ctx.set_source_surface(&image_surface, x, y)?;
        ctx.paint()?;

        Ok(())
    }

    pub fn crop(&self, x: f64, y: f64, width: i32, height: i32) -> anyhow::Result<ImageSurface> {
        let output_surface = ImageSurface::create(Format::ARgb32, width, height)?;
        let output_ctx = Context::new(&output_surface)?;

        output_ctx.set_source_surface(&self.surface, -x, -y)?;
        output_ctx.paint()?;

        Ok(output_surface)
    }

    pub fn crop_to_image(
        &self,
        x: f64,
        y: f64,
        width: u32,
        height: u32,
    ) -> anyhow::Result<DynamicImage> {
        let mut output_surface = self.crop(x, y, width as i32, height as i32)?;
        let output_data = output_surface.data()?;

        let mut data_vec = output_data.to_vec();
        Self::invert_rgba_vec(&mut data_vec);

        let img = RgbaImage::from_vec(width, height, data_vec)
            .expect("Couldn't create image from buffer");

        Ok(DynamicImage::from(img))
    }

    fn invert_rgba_vec(data: &mut Vec<u8>) {
        for i in (0..data.len()).step_by(4) {
            let r = data[i];
            let g = data[i + 1];
            let b = data[i + 2];
            let a = data[i + 3];

            data[i] = b;
            data[i + 1] = g;
            data[i + 2] = r;
            data[i + 3] = a;
        }
    }
}
