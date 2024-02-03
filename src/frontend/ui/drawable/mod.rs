use cairo::ImageSurface;

pub trait Drawable {
    fn draw_active(&self, ctx: &cairo::Context, surface: &ImageSurface);
    fn draw_inactive(&self, ctx: &cairo::Context, surface: &ImageSurface);
    fn draw_final(&self, ctx: &cairo::Context, surface: &ImageSurface);
}
