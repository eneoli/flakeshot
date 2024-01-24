use cairo::ImageSurface;

pub trait Drawable {
    fn draw(&self, ctx: &cairo::Context, surface: &ImageSurface);
}
