use flakeshot::backend::wayland::wayland_screenshot::WaylandScreenshot;

fn main() {
    WaylandScreenshot::create_screenshots().unwrap();
}
