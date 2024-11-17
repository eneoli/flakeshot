use crate::backend::wayland::wayland_error::WaylandError;
use crate::backend::wayland::wayland_screenshot_manager::WaylandScreenshotManager;
use crate::backend::OutputInfo;
use image::DynamicImage::{ImageRgb8, ImageRgba8};
use image::{DynamicImage, RgbImage, RgbaImage};
use std::io::Read;
use tracing::info;
use wayland_client::protocol::wl_shm::Format;
use wayland_output_info::WaylandOutputInfo;

pub mod wayland_error;
pub(crate) mod wayland_frame_meta;
pub(crate) mod wayland_geometry;
pub(crate) mod wayland_output_info;
pub(crate) mod wayland_output_mode;
pub(crate) mod wayland_screenshot_manager;
pub(crate) mod wayland_screenshot_state;
pub(crate) mod wayland_shared_memory;

type FnCreateScreenshot =
    dyn Fn(&mut WaylandScreenshotManager, &WaylandOutputInfo) -> Result<DynamicImage, WaylandError>;

/// The main function of this module.
/// Creates and collects the screenshot of each screen.
///
/// # Example
/// ```no_test
/// use flakeshot::backend::wayland::create_screenshots;
/// use std::fs::File;
/// use image::ImageOutputFormat;
///
/// fn main() {
///     let mut file = File::create("./targets/example_screenshot.png").unwrap();
///     let images = create_screenshots().unwrap();
///
///     // we will only use the first screenshot for this example
///     let first_screen = images.first().unwrap();
///     let image = &first_screen.1;
///
///     image.write_to(&mut file, ImageOutputFormat::Png).unwrap();
/// }
/// ```
pub fn create_screenshots() -> Result<Vec<(OutputInfo, DynamicImage)>, WaylandError> {
    match try_with_portal() {
        Ok(screenshots) => return Ok(screenshots),
        Err(e) => info!("Wayland: {}", e),
    };

    inner_create_screenshots(&manual_create_screenshot)
}

/// A generalized function which iterates through all screens and creates a screenshot of it.
///
/// # Arguments
/// - `create_screenshot_fn`: This function will be called for each screen and it should return the screenshot with the given data of
///                           the screen
fn inner_create_screenshots(
    create_screenshot_fn: &FnCreateScreenshot,
) -> Result<Vec<(OutputInfo, DynamicImage)>, WaylandError> {
    let mut screenshots: Vec<(OutputInfo, DynamicImage)> = vec![];
    let mut manager = WaylandScreenshotManager::new()?;
    let outputs = manager.get_outputs()?.clone();

    for output in outputs {
        let img = create_screenshot_fn(&mut manager, &output)?;
        let output_info = OutputInfo::from(&output);

        screenshots.push((output_info, img));

        manager.next_screen(); // reset current screenshot metadata
    }

    Ok(screenshots)
}

/// This function collects from each screen (a.k.a your monitors) a screenshot
/// and returns it.
fn manual_create_screenshot(
    manager: &mut WaylandScreenshotManager,
    output: &WaylandOutputInfo,
) -> Result<DynamicImage, WaylandError> {
    let queue_handle = manager.get_queue_handle();
    let screenshot_manager = manager.get_zwlr_screencopy_manager_v1()?.clone();

    let frame = screenshot_manager.capture_output(0, &output.output, &queue_handle, ());

    let mut shared_memory = manager.create_shared_memory()?;
    frame.copy(shared_memory.get_buffer());

    manager.await_screenshot()?;

    // read from shared memory
    // data holds our screenshot
    let mut data = vec![];
    shared_memory
        .get_memfile()
        .read_to_end(&mut data)
        .map_err(|_| WaylandError::GenericError("Couldn't read shared memory file"))?;

    let img = {
        let width = shared_memory.width();
        let height = shared_memory.height();
        let format = shared_memory.format();

        image_from_wayland(data, width, height, format)?
    };

    shared_memory.destroy();

    Ok(img)
}

/// This function attempts to create the screenshot by using portals
fn try_with_portal() -> Result<Vec<(OutputInfo, DynamicImage)>, WaylandError> {
    let screenshot = super::portal::create_screenshot()?;

    let cropper = move |_manager: &mut WaylandScreenshotManager,
                        output: &WaylandOutputInfo|
          -> Result<DynamicImage, WaylandError> {
        let output = OutputInfo::from(output);
        Ok(screenshot.crop_imm(
            output.x as u32,
            output.y as u32,
            output.width.into(),
            output.height.into(),
        ))
    };

    inner_create_screenshots(&cropper)
}

// Transforms the buffer containing our image from the wayland compositor into a `image::DynamicImage`.
fn image_from_wayland(
    data: Vec<u8>,
    width: u32,
    height: u32,
    format: Format,
) -> Result<DynamicImage, WaylandError> {
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
