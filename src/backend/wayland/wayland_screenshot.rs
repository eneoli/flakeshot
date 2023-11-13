use std::io::Read;
use std::path::Path;
use wayland_client::Connection;
use wayland_client::protocol::wl_shm::Format::{Abgr8888, Argb8888, Xbgr8888};
use crate::backend::wayland::wayland_shared_memory::{WaylandSharedMemory};
use crate::backend::wayland::wayland_error::WaylandError;
use crate::backend::wayland::wayland_screenshot_state::WaylandScreenshotState;

pub struct WaylandScreenshot {}

impl WaylandScreenshot {
    pub fn create_screenshots() -> anyhow::Result<()> {
        let conn = Connection::connect_to_env()?;

        let display = conn.display();
        let mut queue = conn.new_event_queue();
        let queue_handle = queue.handle();

        // attach to globals
        display.get_registry(&queue_handle, ());

        let mut state = WaylandScreenshotState::default();

        queue.roundtrip(&mut state)?;

        loop {
            queue.blocking_dispatch(&mut state)?;

            // spin util we got all outputs from wayland
            if !state.outputs_fetched {
                continue;
            }

            let screenshot_manager = state.zwlr_screencopy_manager_v1
                .as_ref()
                .ok_or(WaylandError::NoScreenshotManager)?;


            let frame = screenshot_manager.capture_output(
                0,
                &state.outputs.last_mut().unwrap().output,
                &queue_handle,
                (),
            );

            while state.current_frame.is_none() {
                queue.blocking_dispatch(&mut state).unwrap();
            }

            let (width, height, stride) = {
                let current_frame = state.current_frame.as_ref().unwrap();
                let width = current_frame.width.clone();
                let height = current_frame.height.clone();
                let stride = current_frame.stride.clone();

                (width, height, stride)
            };

            let shared_memory = WaylandSharedMemory::new(
                state.wl_shm
                    .as_ref()
                    .ok_or(WaylandError::NoShmBind)?,
                &queue_handle,
                width,
                height,
                stride,
                Xbgr8888,
            )?;

            frame.copy(shared_memory.get_buffer());

            while !state.screenshot_ready {
                queue.blocking_dispatch(&mut state).unwrap();
            }

            let mut data = vec![];

            shared_memory.get_memfile().read_to_end(&mut data)?;

            image::save_buffer(
                &Path::new("/home/oliver/screen.png"),
                data.as_slice(),
                width,
                height,
                image::ColorType::Rgba8,
            )?;

            //shared_memory.destroy();

            break; // done
        }

        Ok(())
    }
}