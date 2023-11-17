use std::io::Read;
use image::{DynamicImage, RgbaImage, RgbImage};
use image::DynamicImage::{ImageRgb8, ImageRgba8};
use wayland_client::{Connection};
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


pub async fn create_screenshots() -> anyhow::Result<Vec<(OutputInfo, DynamicImage)>> {
    let conn = Connection::connect_to_env()?;

    let display = conn.display();
    let mut queue = conn.new_event_queue();
    let queue_handle = queue.handle();

    // attach to globals
    display.get_registry(&queue_handle, ());

    let mut state = WaylandScreenshotState::default();

    queue.roundtrip(&mut state)?;

    // spin util we got all outputs from wayland
    //await_event_queue(|state| {false}, queue, state).await;
    while !state.outputs_fetched {
        queue.blocking_dispatch(&mut state)?;
    }

    let screenshot_manager = state.zwlr_screencopy_manager_v1
        .as_ref()
        .ok_or(WaylandError::NoScreenshotManager)?
        .clone();


    let mut screenshots: Vec<(OutputInfo, DynamicImage)> = vec![];
    for i in 0..state.outputs.len() {
        let output = &state.outputs[i];

        let frame = screenshot_manager.capture_output(
            0,
            &output.output,
            &queue_handle,
            (),
        );

        // spin until screenshot ready
        while state.current_frame.is_none() {
            queue.blocking_dispatch(&mut state)?;
        }

        let (width, height, stride) = {
            let current_frame = state.current_frame
                .as_ref()
                .ok_or(WaylandError::BrokenState)?;

            let width = current_frame.width;
            let height = current_frame.height;
            let stride = current_frame.stride;

            (width, height, stride)
        };

        let format = state.current_frame
            .as_ref()
            .ok_or(WaylandError::BrokenState)?
            .format
            .ok_or(WaylandError::MissingFormat)?;

        let mut shared_memory = WaylandSharedMemory::new(
            state.wl_shm
                .as_ref()
                .ok_or(WaylandError::NoShmBind)?,
            &queue_handle,
            width,
            height,
            stride,
            format,
        )?;

        frame.copy(shared_memory.get_buffer());

        // spin until screenshot copied into shared buffer
       // await_event_queue(|state| { false }, &mut queue, &mut state).await;

        while !state.screenshot_ready {
            queue.blocking_dispatch(&mut state)?;
        }

        // read from shared memory
        // data holds our screenshot
        let mut data = vec![];
        shared_memory.get_memfile().read_to_end(&mut data)?;

        let output_info = OutputInfo::try_from(&state.outputs[i])?;
        let img = image_from_wayland(data, width, height, format)?;

        screenshots.push((output_info, img));

        shared_memory.destroy();
        state.next_screen(); // reset current screenshot metadata
    }

    Ok(screenshots)
}

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
        _ => unimplemented!("Your wayland compositor returned an unsupported buffer format: {:#?}", format),
    };

    Ok(result)
}