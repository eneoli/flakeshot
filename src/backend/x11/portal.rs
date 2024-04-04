//! The screenshot creator by making use of the [org.freedesktop.portal.Screenshot] portal.
//!
//! [org.freedesktop.portal.Screenshot]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html
use anyhow::Result;
use ashpd::desktop::screenshot::Screenshot;
use image::{io::Reader, DynamicImage};
use tokio::runtime::Runtime;

use super::ScreenshotCreator;

/// Some errors which could occur while trying to create a screenshot from the portals.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Ashpd(#[from] ashpd::Error),

    #[error("Couldn't open screenshot file from dbus: {0}")]
    IO(#[from] std::io::Error),

    #[error("Couldn't decode screenshot file from dbus: {0}")]
    ImageDecode(#[from] image::error::ImageError),
}

#[derive(Debug)]
pub struct PortalScreenshot {
    /// Portal gives us a screenshot from all monitors. We just crop it later to the suitable
    /// monitor for each request in `get_image`.
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
    fn create_screenshot(
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

    fn get_name(&self) -> &'static str {
        "Portal-Screenshot"
    }
}

async fn request_screenshot() -> ashpd::Result<Screenshot> {
    Screenshot::request()
        .interactive(false)
        .send()
        .await?
        .response()
}
