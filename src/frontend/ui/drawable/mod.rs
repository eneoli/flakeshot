use gtk4::cairo::Context;

pub trait Drawable {
    fn draw(&self, ctx: &Context);
}
