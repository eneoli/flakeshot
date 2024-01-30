mod command;
mod error;

pub use command::Command;
pub use error::Error;

use anyhow::Context;
use relm4::Sender;
use std::{fs::File, io::Cursor};
use tracing::{error, info};

use image::{ImageBuffer, Rgba};
use ksni;

use crate::get_xdg;

const LOCK_FILE: &str = "tray.lock";

#[tracing::instrument]
pub async fn start(sender: Sender<Command>) {
    let _lock_file = match acquire_lock() {
        Ok(Some(lock_file)) => lock_file,
        Ok(None) => return,
        Err(e) => {
            sender.send(Command::Notify(format!("{}", e))).unwrap();
            return;
        }
    };

    let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    ksni::run_async(Tray::new(sender), rx)
        .await
        .expect("Couldn't run tray");
}

#[derive(Debug)]
pub struct Tray {
    sender: Sender<Command>,
    icon: ksni::Icon,
}

impl Tray {
    fn new(sender: Sender<Command>) -> Self {
        let rgba_image = get_tray_image();
        let (width, height) = rgba_image.dimensions();

        let mut data: Vec<u8> = rgba_image.into_vec();

        // rgba => argb
        data.chunks_mut(4).for_each(|pixel| pixel.rotate_right(1));

        Self {
            sender,
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
        self.sender.send(Command::CreateScreenshot).unwrap();
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

/// If no error occured: Returns the lock-file (if available), otherwise `None` if the lock file
/// couldn't be aquired.
/// Otherwise the error will be returned.
#[tracing::instrument]
pub fn acquire_lock() -> anyhow::Result<Option<File>> {
    let lock_file_path = get_xdg().place_runtime_file(LOCK_FILE).unwrap();

    let lock_file = File::create(lock_file_path).context("Create tray lock file")?;
    if let Err(err) = rustix::fs::flock(
        &lock_file,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    ) {
        let daemon_already_exists = err == rustix::io::Errno::WOULDBLOCK;

        if daemon_already_exists {
            info!("Tray is already running");
            return Ok(None);
        } else {
            error!("Couldn't acquire lock: {}", err);
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
