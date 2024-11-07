//! The screenshot creator by making use of the [org.freedesktop.portal.Screenshot] portal.
//!
//! [org.freedesktop.portal.Screenshot]: https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html

use anyhow::Result;
use ashpd::desktop::screenshot::Screenshot;
use image::{DynamicImage, ImageReader};
use tokio::runtime::Runtime;

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

async fn request_screenshot() -> ashpd::Result<Screenshot> {
    Screenshot::request()
        .interactive(false)
        .send()
        .await?
        .response()
}

/// We try to use the screenshot portals first
pub fn create_screenshot() -> Result<DynamicImage, Error> {
    let rt = Runtime::new()?;
    let screenshot = rt.block_on(request_screenshot())?;

    let file_path = screenshot
        .uri()
        .to_file_path()
        .expect("The screenshot portal didn't return a file uri!");

    let screenshot = ImageReader::open(file_path)?.decode()?;

    Ok(screenshot)
}
