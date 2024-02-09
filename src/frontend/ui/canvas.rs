use gtk4::cairo::{Context, Filter, Format, ImageSurface};
use image::{DynamicImage, RgbaImage};

pub trait CanvasDrawable {
    fn draw(&self, ctx: &Context, surface: &ImageSurface);
}

#[derive(Debug)]
pub struct Canvas {
    surface: ImageSurface,
    original: ImageSurface,
}

impl Canvas {
    pub fn new(width: i32, height: i32) -> anyhow::Result<Self> {
        Ok(Canvas {
            surface: ImageSurface::create(Format::ARgb32, width, height)?,
            original: ImageSurface::create(Format::ARgb32, width, height)?,
        })
    }

    pub fn from_original(&self) -> Self {
        Canvas {
            surface: self.original.clone(),
            original: self.original.clone(),
        }
    }

    pub fn width(&self) -> i32 {
        self.surface.width()
    }

    pub fn height(&self) -> i32 {
        self.surface.height()
    }

    pub fn clear(&mut self) -> anyhow::Result<()> {
        let ctx = Context::new(&self.surface)?;
        ctx.set_source_surface(&self.original, 0.0, 0.0)?;
        ctx.paint()?;

        Ok(())
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        let original_ctx = Context::new(&self.original)?;
        original_ctx.set_source_surface(&self.surface, 0.0, 0.0)?;
        original_ctx.paint()?;

        Ok(())
    }

    pub fn render_drawable(&mut self, drawable: &dyn CanvasDrawable) {
        let ctx = Context::new(&self.surface).unwrap();
        drawable.draw(&ctx, &self.surface);
    }

    pub fn stamp_image(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        image: &DynamicImage,
    ) -> anyhow::Result<()> {
        let ctx = Context::new(&self.surface)?;

        let mut image_bytes = Vec::from(image.as_bytes());

        // reverse RGB, keep Alpha
        for i in (0..image_bytes.len()).step_by(4) {
            image_bytes[i..i + 3].reverse();
        }

        let image_surface = ImageSurface::create_for_data(
            image_bytes,
            Format::ARgb32,
            image.width() as i32,
            image.height() as i32,
            Format::stride_for_width(Format::ARgb32, image.width())?,
        )?;

        ctx.scale(width / image.width() as f64, height / image.height() as f64);
        ctx.set_source_surface(&image_surface, x, y)?;
        ctx.source().set_filter(Filter::Best);
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

        let data_vec = output_data.to_vec();
        let mut img = RgbaImage::from_vec(width, height, data_vec)
            .expect("Couldn't create image from buffer");

        // Reverse RGB, keep Alpha
        for pixel in img.pixels_mut() {
            pixel.0[0..3].reverse();
        }

        Ok(DynamicImage::from(img))
    }
}
