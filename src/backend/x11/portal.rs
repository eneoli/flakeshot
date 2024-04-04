use anyhow::Result;
use ashpd::desktop::screenshot::Screenshot;
use image::{io::Reader, DynamicImage};
use tokio::runtime::Runtime;

use super::ScreenshotCreator;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error occured from ashpd: {0}")]
    Ashpd(#[from] ashpd::Error),

    #[error("")]
    DbusUnknownFilePath,

    #[error("Couldn't open screenshot file from dbus: {0}")]
    IO(#[from] std::io::Error),

    #[error("Couldn't decode screenshot file from dbus: {0}")]
    ImageDecode(#[from] image::error::ImageError),
}

#[derive(Debug)]
pub struct PortalScreenshot {
    screenshot: DynamicImage,
}

impl PortalScreenshot {
    pub fn new() -> Result<Self, Error> {
        let rt = Runtime::new()?;
        let screenshot = rt.block_on(request_screenshot())?;

        let file_path = screenshot
            .uri()
            .to_file_path()
            .expect("The screenshot portal didn't return a file uri!");

        let screenshot = Reader::open(file_path)?.decode()?;

        Ok(Self { screenshot })
    }
}

impl ScreenshotCreator for PortalScreenshot {
    fn get_image(
        &self,
        _conn: &x11rb::rust_connection::RustConnection,
        _screen: &x11rb::protocol::xproto::Screen,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<DynamicImage, super::Error> {
        Ok(self
            .screenshot
            .crop_imm(x as u32, y as u32, width as u32, height as u32))
    }
}

async fn request_screenshot() -> ashpd::Result<Screenshot> {
    Screenshot::request()
        .interactive(false)
        .send()
        .await?
        .response()
}
