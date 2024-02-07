pub mod error;

use std::{fs::File, io::Cursor};

use anyhow::Context;
use image::{ImageBuffer, Rgba};
use ksni;
use relm4::Sender;

use crate::{frontend::window::main_window::Command, get_xdg};

use self::error::Error;

const LOCK_FILENAME: &str = "flakeshot.lock";

#[derive(Debug)]
struct Tray {
    icon: ksni::Icon,
    sender: Sender<Command>,
}

impl Tray {
    pub fn new(sender: Sender<Command>) -> Self {
        let rgba_image = get_tray_image();
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
            sender,
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
        self.sender
            .send(Command::Gui)
            .expect("Couldn't send gui command");
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        let sender = self.sender.clone();

        vec![StandardItem {
            label: "Quit".into(),
            activate: Box::new(move |_| {
                sender
                    .send(Command::Quit)
                    .expect("Couldn't send quit command")
            }),
            ..Default::default()
        }
        .into()]
    }
}

pub async fn start(sender: Sender<Command>) {
    let _lock_file = match acquire_lock() {
        Ok(Some(lock_file)) => lock_file,
        Ok(None) => return,
        Err(e) => {
            panic!("Couldn't start the tray: {}", e);
        }
    };

    let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    ksni::run_async(Tray::new(sender), rx)
        .await
        .expect("Couldn't run tray");
}

pub fn acquire_lock() -> anyhow::Result<Option<File>> {
    let lock_file_path = get_xdg().place_runtime_file(LOCK_FILENAME).unwrap();

    let lock_file = File::create(lock_file_path).context("Create tray lock file")?;
    if let Err(err) = rustix::fs::flock(
        &lock_file,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    ) {
        let tray_already_exists = err == rustix::io::Errno::WOULDBLOCK;

        if tray_already_exists {
            return Ok(None);
        } else {
            return Err(Error::AcquireSocket(err).into());
        }
    }

    Ok(Some(lock_file))
}

fn get_tray_image() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let cursor = {
        let image_bytes = include_bytes!("../../assets/flakeshot_logo_dpi_96.png");
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

    /// Makes sure that the tray image will be always correctly loaded
    #[test]
    fn test_get_tray_image() {
        get_tray_image();
    }
}
