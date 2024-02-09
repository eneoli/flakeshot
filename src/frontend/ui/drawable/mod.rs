use gtk4::cairo::{Context, ImageSurface};

pub trait Drawable: std::fmt::Debug {
    fn draw_active(&self, ctx: &Context, surface: &ImageSurface);
    fn draw_inactive(&self, ctx: &Context, surface: &ImageSurface);
    fn draw_final(&self, ctx: &Context, surface: &ImageSurface);
}
