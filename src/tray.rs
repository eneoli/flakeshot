use std::io::Cursor;

use image::{ImageBuffer, Rgba};
use ksni;

#[derive(Debug)]
struct Tray {
    icon: ksni::Icon,
}

impl Tray {
    pub fn new() -> Self {
        let rgba_image = get_tray_image();
        let (width, height) = rgba_image.dimensions();
        let data = rgba_image
            .pixels()
            // rgba => argb
            .map(|pixel| Rgba::from([pixel[3], pixel[0], pixel[1], pixel[2]]))
            .fold(
                Vec::with_capacity((width * height) as usize),
                |mut prev, pixel| {
                    prev.extend_from_slice(&pixel.0);
                    prev
                },
            );

        Self {
            icon: ksni::Icon {
                width: width as i32,
                height: height as i32,
                data,
            },
        }
    }
}

impl ksni::Tray for Tray {
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![self.icon.clone()]
    }

    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        crate::start_gui();
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![StandardItem {
            label: "Quit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into()]
    }
}

pub fn start() {
    tracing::debug!("Starting tray");

    ksni::spawn(Tray::new()).expect("Couldn't spawn tray.");

    loop {
        std::thread::park();
    }
}

fn get_tray_image() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let cursor = {
        let image_bytes = include_bytes!("../assets/flakeshot_logo_dpi_96.png");
        Cursor::new(image_bytes)
    };
    image::io::Reader::with_format(cursor, image::ImageFormat::Png)
        .decode()
        .unwrap()
        .to_rgba8()
}

#[cfg(test)]
mod tests {
    use super::get_tray_image;

    /// Makes sure that the tray image is always correctly loaded
    #[test]
    fn test_get_tray_image() {
        get_tray_image();
    }
}
