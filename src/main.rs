use std::fs::File;

use flakeshot::backend;

fn run() -> Result<(), ()> {
    Ok(())
}

fn main() {
    let images = backend::get_images().unwrap();
    let yes = images.first().unwrap();

    let mut file = File::create("/tmp/test.png").unwrap();
    yes.write_to(&mut file, image::ImageFormat::Png).unwrap();
}
