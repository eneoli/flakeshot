pub trait Drawable {
    fn draw(&self, ctx: &cairo::Context);
}
