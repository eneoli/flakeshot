use std::fs::File;

use flakeshot::backend;
use image::ImageFormat;

async fn run() -> Result<(), ()> {
    Ok(())
}

#[tokio::main]
async fn main() {
    let images = backend::get_images().unwrap();
    let image = images.first().unwrap();

    let mut file = File::create("/tmp/maybe.png").unwrap();
    image.write_to(&mut file, ImageFormat::Png).unwrap();
    run().await.unwrap();
}
