use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use image::{DynamicImage, RgbaImage, RgbImage};
use image::DynamicImage::{ImageRgb8, ImageRgba8};
use wayland_client::{Connection, EventQueue};
use wayland_client::protocol::wl_shm::Format;
use crate::backend::OutputInfo;
use crate::backend::wayland::wayland_shared_memory::{WaylandSharedMemory};
use crate::backend::wayland::wayland_error::WaylandError;
use crate::backend::wayland::wayland_screenshot_state::WaylandScreenshotState;

pub mod wayland_error;
pub(crate) mod wayland_screenshot_state;
pub(crate) mod wayland_shared_memory;
pub(crate) mod wayland_output_info;
pub(crate) mod wayland_frame_meta;
pub(crate) mod wayland_geometry;
pub(crate) mod wayland_output_mode;

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
pub async fn create_screenshots() -> anyhow::Result<Vec<(OutputInfo, DynamicImage)>> {
    let conn = Connection::connect_to_env()?;

    let display = conn.display();

    let queue_mutex: Arc<Mutex<EventQueue<WaylandScreenshotState>>> = Arc::new(Mutex::new(conn.new_event_queue()));

    let queue_handle = {
        let queue = &mut *queue_mutex.lock().unwrap();
        queue.handle()
    };

    // attach to globals
    display.get_registry(&queue_handle, ());

    let state_mutex = Arc::new(Mutex::new(WaylandScreenshotState::default()));

    {
        let mut state = &mut *state_mutex.lock().unwrap();
        let queue = &mut *queue_mutex.lock().unwrap();

        queue.roundtrip(&mut state)?;
    }

    // spin util we got all outputs from wayland
    await_queue_events(
        |state| { state.outputs_fetched },
        queue_mutex.clone(),
        state_mutex.clone(),
    ).await?;

    let screenshot_manager = {
        let state = &mut *state_mutex.lock().unwrap();

        state.zwlr_screencopy_manager_v1
            .as_ref()
            .ok_or(WaylandError::NoScreenshotManager)?
            .clone()
    };

    let num_of_outputs = {
        let state = &mut *state_mutex.lock().unwrap();
        state.outputs.len()
    };

    let mut screenshots: Vec<(OutputInfo, DynamicImage)> = vec![];
    for i in 0..num_of_outputs {
        let output = {
            let state = &mut *state_mutex.lock().unwrap();
            &state.outputs[i].clone()
        };

        let frame = screenshot_manager.capture_output(
            0,
            &output.output,
            &queue_handle,
            (),
        );

        // spin until screenshot ready
        await_queue_events(
            |state| { state.current_frame.is_some() },
            queue_mutex.clone(),
            state_mutex.clone(),
        ).await?;

        let (width, height, stride) = {
            let state = &mut *state_mutex.lock().unwrap();

            let current_frame = state.current_frame
                .as_ref()
                .ok_or(WaylandError::BrokenState)?;

            let width = current_frame.width;
            let height = current_frame.height;
            let stride = current_frame.stride;

            (width, height, stride)
        };

        let format = {
            let state = &mut *state_mutex.lock().unwrap();

            state.current_frame
                .as_ref()
                .ok_or(WaylandError::BrokenState)?
                .format
                .ok_or(WaylandError::MissingFormat)?
        };

        let mut shared_memory = {
            let state = &mut *state_mutex.lock().unwrap();

            WaylandSharedMemory::new(
                state.wl_shm
                    .as_ref()
                    .ok_or(WaylandError::NoShmBind)?,
                &queue_handle,
                width,
                height,
                stride,
                format,
            )?
        };

        frame.copy(shared_memory.get_buffer());

        // spin until screenshot copied into shared buffer
        await_queue_events(
            |state| { state.screenshot_ready },
            queue_mutex.clone(),
            state_mutex.clone(),
        ).await?;

        // read from shared memory
        // data holds our screenshot
        let mut data = vec![];
        shared_memory.get_memfile().read_to_end(&mut data)?;

        let output_info = OutputInfo::try_from({
            let state = &mut *state_mutex.lock().unwrap();
            &state.outputs[i].clone()
        })?;
        let img = image_from_wayland(data, width, height, format)?;

        screenshots.push((output_info, img));

        shared_memory.destroy();

        let state = &mut *state_mutex.lock().unwrap();
        state.next_screen(); // reset current screenshot metadata
    }

    Ok(screenshots)
}

// Transforms the buffer containing our image from the wayland compositor into a `image::DynamicImage`.
fn image_from_wayland(data: Vec<u8>, width: u32, height: u32, format: Format) -> anyhow::Result<DynamicImage> {
    let result = match format {
        Format::Argb8888 |
        Format::Abgr8888 |
        Format::Xrgb8888 |
        Format::Xbgr8888 => {
            ImageRgba8(
                RgbaImage::from_vec(width, height, data)
                    .ok_or(WaylandError::ConvertImageFailed)?
            )
        }
        Format::Bgr888 => {
            ImageRgb8(
                RgbImage::from_vec(width, height, data)
                    .ok_or(WaylandError::ConvertImageFailed)?
            )
        }
        _ => unimplemented!("Your wayland compositor returned an unsupported buffer format: {:#?}.\
        You might want to open an issue on GitHub.", format),
    };

    Ok(result)
}

// This is an utility function that allows to asynchronously wait for an event.
// We spawn a new thread that polls the queue using `poll_queue`.
// Also this functions timeouts after 2 minutes when `poll_queue` does not return.
async fn await_queue_events<T: 'static + Send>(
    until: impl Fn(&T) -> bool + 'static + Send,
    queue_mutex: Arc<Mutex<EventQueue<T>>>,
    state_mutex: Arc<Mutex<T>>,
) -> anyhow::Result<()> {
    let timeout_result = tokio::time::timeout(
        Duration::from_secs(120),
        tokio::spawn(
            async {
                poll_queue(until, queue_mutex, state_mutex)
            }
        )).await;

    match timeout_result {
        Ok(Ok(Ok(()))) => Ok(()),
        Ok(Ok(Err(_))) => Err(WaylandError::EventQueuePollingError.into()), // error in poll_queue
        Ok(Err(_)) => Err(WaylandError::ThreadSpawnFailed.into()), // error in tokio::spawn
        Err(_) => Err(WaylandError::EventQueueTimeout.into()), // Future Timeout
    }
}

// polls queue until the function 'until' returns true.
// Blocks the current thread
fn poll_queue<T>(
    until: impl Fn(&T) -> bool,
    queue_mutex: Arc<Mutex<EventQueue<T>>>,
    state_mutex: Arc<Mutex<T>>,
) -> anyhow::Result<()> {
    let queue = &mut *queue_mutex.lock().unwrap();
    let state = &mut *state_mutex.lock().unwrap();

    while !until(state) {
        queue.blocking_dispatch(state)?;
    }

    Ok(())
}