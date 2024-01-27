use std::io::Cursor;

use image::Rgba;
use ksni;

#[derive(Debug)]
struct Tray {
    icon: ksni::Icon,
}

impl Tray {
    pub fn new() -> Self {
        let rgba_image = {
            let cursor = {
                let image_bytes = include_bytes!("../assets/flakeshot_logo_dpi_96.png");
                Cursor::new(image_bytes)
            };
            image::io::Reader::with_format(cursor, image::ImageFormat::Png)
                .decode()
                .unwrap()
                .to_rgba8()
        };

        let (width, height) = rgba_image.dimensions();

        let mut data: Vec<u8> = rgba_image.into_vec();

        // rgba => argb
        data.chunks_mut(4).for_each(|pixel| pixel.rotate_right(1));

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
