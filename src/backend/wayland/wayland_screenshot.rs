use std::io::Read;
use std::os::fd::AsFd;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use wayland_client::Connection;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_shm;
use crate::backend::wayland::shared_memory::create_shm_file;
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

        let mut memfile = create_shm_file("/tmp/flakeshot", 2160 * 3840 * 4)?; // TODO stride and resolution

        let buffer: WlBuffer = {
            // setup shm
            let wl_shm = state.wl_shm
                .as_ref()
                .ok_or(WaylandError::NoShmBind)?;

            let shm_pool = wl_shm.create_pool(
                memfile.as_fd(),
                3840 * 2160 * 4,
                &queue_handle,
                (),
            );

            shm_pool.create_buffer(
                0,
                3840,
                2160,
                3840 * 4,
                wl_shm::Format::Xbgr8888, // TODO check for right format
                &queue_handle,
                (),
            )
        };

        loop {
            queue.blocking_dispatch(&mut state)?;

            // spin util we got all outputs from wayland
            if !state.outputs_fetched {
                continue;
            }

            // TODO can we do it better?
            {
                let screenshot_manager = state.zwlr_screencopy_manager_v1
                    .as_ref()
                    .ok_or(WaylandError::NoScreenshotManager)?;


                let frame = screenshot_manager.capture_output(
                    0,
                    &state.outputs.last_mut().unwrap().output,
                    &queue_handle,
                    (),
                );

                frame.copy(&buffer);
            }

            while !state.screenshot_ready {
                queue.blocking_dispatch(&mut state).unwrap();
            }

            sleep(Duration::from_secs(5)); // TODO use events instead of sleep

            let mut data = vec![];

            memfile.read_to_end(&mut data)?;

            image::save_buffer(
                &Path::new("/home/oliver/screen.png"),
                data.as_slice(),
                3840,
                2160,
                image::ColorType::Rgba8,
            )?;

            break; // done
        }

        Ok(())
    }
}