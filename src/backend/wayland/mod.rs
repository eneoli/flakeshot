use crate::backend::wayland::wayland_error::WaylandError;
use crate::backend::wayland::wayland_screenshot_manager::WaylandScreenshotManager;
use crate::backend::OutputInfo;
use image::DynamicImage::{ImageRgb8, ImageRgba8};
use image::{DynamicImage, RgbImage, RgbaImage};
use std::io::Read;
use wayland_client::protocol::wl_shm::Format;

pub mod wayland_error;
pub(crate) mod wayland_frame_meta;
pub(crate) mod wayland_geometry;
pub(crate) mod wayland_output_info;
pub(crate) mod wayland_output_mode;
pub(crate) mod wayland_screenshot_manager;
pub(crate) mod wayland_screenshot_state;
pub(crate) mod wayland_shared_memory;

/// The main function of this module.
///
/// This function collects, from each screen (a.k.a your monitors) a screenshot
/// and returns it.
///
/// # Example
/// ```no_run
/// use flakeshot::backend::wayland::create_screenshots;
/// use std::fs::File;
/// use image::ImageOutputFormat;
///
/// #[tokio::main]
/// async fn main() {
///     let mut file = File::create("./targets/example_screenshot.png").unwrap();
///     let images = create_screenshots().await.unwrap();
///
///     // we will only use the first screenshot for this example
///     let first_screen = images.first().unwrap();
///     let image = &first_screen.1;
///
///     image.write_to(&mut file, ImageOutputFormat::Png).unwrap();
/// }
/// ```
pub fn create_screenshots() -> anyhow::Result<Vec<(OutputInfo, DynamicImage)>> {
    let mut manager = WaylandScreenshotManager::new()?;
    let queue_handle = manager.get_queue_handle();

    let screenshot_manager = manager.get_zwlr_screencopy_manager_v1()?.clone();

    let num_outputs = { manager.get_outputs()?.len() };

    let mut screenshots: Vec<(OutputInfo, DynamicImage)> = vec![];
    for i in 0..num_outputs {
        let img = {
            let frame = {
                let output = &manager.get_outputs()?[i];

                screenshot_manager.capture_output(0, &output.output, &queue_handle, ())
            };

            let mut shared_memory = manager.create_shared_memory()?;
            frame.copy(shared_memory.get_buffer());

            manager.await_screenshot()?;

            // read from shared memory
            // data holds our screenshot
            let mut data = vec![];
            shared_memory.get_memfile().read_to_end(&mut data)?;

            let img = {
                let width = shared_memory.width();
                let height = shared_memory.height();
                let format = shared_memory.format();

                image_from_wayland(data, width, height, format)?
            };

            shared_memory.destroy();

            img
        };

        let output_info = OutputInfo::try_from(&manager.get_outputs()?[i])?;

        screenshots.push((output_info, img));

        manager.next_screen(); // reset current screenshot metadata
    }

    Ok(screenshots)
}

// Transforms the buffer containing our image from the wayland compositor into a `image::DynamicImage`.
fn image_from_wayland(
    data: Vec<u8>,
    width: u32,
    height: u32,
    format: Format,
) -> anyhow::Result<DynamicImage> {
    let result = match format {
        Format::Argb8888 | Format::Abgr8888 | Format::Xrgb8888 | Format::Xbgr8888 => ImageRgba8(
            RgbaImage::from_vec(width, height, data).ok_or(WaylandError::ConvertImageFailed)?,
        ),
        Format::Bgr888 => ImageRgb8(
            RgbImage::from_vec(width, height, data).ok_or(WaylandError::ConvertImageFailed)?,
        ),
        _ => unimplemented!(
            "Your wayland compositor returned an unsupported buffer format: {:#?}.\
        You might want to open an issue on GitHub.",
            format
        ),
    };

    Ok(result)
}
