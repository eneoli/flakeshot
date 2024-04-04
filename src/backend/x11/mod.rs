//! Backend implementation for X11.
mod fallback;
mod portal;

use image::DynamicImage;

use tracing::warn;
use x11rb::{connection::Connection, protocol::xproto::Screen, rust_connection::RustConnection};

use self::{fallback::Fallback, portal::PortalScreenshot};

use super::{MonitorInfo, OutputInfo};

/// A general enum with possible errors as values which can occur while
/// operating with the xorg-server.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Couldn't connect to the xorg server: {0}")]
    ConnectError(#[from] x11rb::errors::ConnectError),

    #[error("The connection broke with the xorg server: {0}")]
    ConnectionError(#[from] x11rb::errors::ConnectionError),

    #[error("Couldn't request an image from the xorg-server: {0}")]
    ReplyError(#[from] x11rb::errors::ReplyError),

    #[error(transparent)]
    StringUtf8(#[from] std::string::FromUtf8Error),

    #[error("An error occured while trying to create a screenshot through the portals: {0}")]
    PortalError(#[from] portal::Error),
}

/// The main function of this module.
///
/// This function collects, from each screen (a.k.a your monitors) a screenshot
/// and returns it.
///
/// # Example
/// ```no_test
/// use flakeshot::backend::x11::get_images;
/// use std::fs::File;
/// use image::ImageOutputFormat;
///
/// fn main() {
///     let mut file = File::create("./targets/example_screenshot.png").unwrap();
///     let images = get_images().unwrap();
///
///     // we will only use the first screenshot for this example
///     let first_screen = images.first().unwrap();
///     let image = &first_screen.1;
///
///     image.write_to(&mut file, ImageOutputFormat::Png).unwrap();
/// }
/// ```
pub fn create_screenshots() -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    let creators = [PortalScreenshot::new()?];

    for creator in creators {
        match try_create_screenshots_with(&creator) {
            Ok(screenshots) => return Ok(screenshots),
            Err(err) => warn!(
                "Couldn't create screenshots from '{}': {}. Trying other creator...",
                creator.get_name(),
                err
            ),
        }
    }

    return try_create_screenshots_with(&Fallback);
}

fn try_create_screenshots_with(
    creator: &impl ScreenshotCreator,
) -> Result<Vec<(OutputInfo, image::DynamicImage)>, Error> {
    use x11rb::protocol::randr::ConnectionExt;

    let (conn, _) = x11rb::connect(None)?;
    let setup = conn.setup();

    let mut images = Vec::with_capacity(setup.roots.len());

    for screen in &setup.roots {
        let monitors = conn.randr_get_monitors(screen.root, true)?.reply()?;

        for monitor in &monitors.monitors {
            assert!(
                monitor.outputs.len() == 1,
                "We currently support only one output for each monitor. Please create an issue if you encounter this assert."
            );

            let image = creator.create_screenshot(
                &conn,
                screen,
                monitor.x,
                monitor.y,
                monitor.width,
                monitor.height,
            )?;
            let monitor_info = {
                let screen_resources = conn
                    .randr_get_screen_resources_current(screen.root)?
                    .reply()?;

                let output_name = {
                    let output_info = conn
                        .randr_get_output_info(
                            monitor.outputs[0],
                            screen_resources.config_timestamp,
                        )?
                        .reply()?;

                    String::from_utf8(output_info.name)?
                };

                MonitorInfo::X11 { name: output_name }
            };

            let output_info = OutputInfo {
                id: screen.root,

                width: monitor.width,
                height: monitor.height,
                x: monitor.x,
                y: monitor.y,

                monitor_info,
            };

            images.push((output_info, image.clone()));
        }
    }

    Ok(images)
}

/// Represents a screenshot creator which should create and return the screenshot of the provided screen.
trait ScreenshotCreator {
    /// The creator which implements this function should return, if it succeeds, the screenshot with the provided meta data
    /// and return it.
    fn create_screenshot(
        &self,
        conn: &RustConnection,
        screen: &Screen,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Result<DynamicImage, Error>;

    /// Returns the name of the creator. (In general it's the name of the struct which implements this.)
    fn get_name(&self) -> &'static str;
}
