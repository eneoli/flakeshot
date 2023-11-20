use crate::backend::wayland::wayland_error::WaylandError;
use crate::backend::wayland::wayland_output_info::WaylandOutputInfo;
use crate::backend::wayland::wayland_screenshot_state::WaylandScreenshotState;
use crate::backend::wayland::wayland_shared_memory::WaylandSharedMemory;
use std::time::SystemTime;
use wayland_client::{Connection, EventQueue, QueueHandle};
use wayland_protocols_wlr::screencopy::v1::client::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1;

pub struct WaylandScreenshotManager {
    connection: Connection,
    queue: EventQueue<WaylandScreenshotState>,
    state: WaylandScreenshotState,
}

impl WaylandScreenshotManager {
    pub fn new() -> anyhow::Result<WaylandScreenshotManager> {
        let connection = Connection::connect_to_env()?;

        let mut queue = {
            let display = connection.display();

            let queue = connection.new_event_queue();
            let queue_handle = queue.handle();

            // attach to globals
            display.get_registry(&queue_handle, ());

            queue
        };

        let mut state = WaylandScreenshotState::default();

        queue.roundtrip(&mut state)?;

        Ok(Self {
            connection,
            state,
            queue,
        })
    }

    pub fn get_queue_handle(&self) -> QueueHandle<WaylandScreenshotState> {
        self.queue.handle()
    }
    pub fn get_zwlr_screencopy_manager_v1(&mut self) -> anyhow::Result<&ZwlrScreencopyManagerV1> {
        self.poll_queue_until(|state| state.zwlr_screencopy_manager_v1.is_some())?;

        Ok(self.state.zwlr_screencopy_manager_v1.as_ref().unwrap())
    }

    pub fn get_outputs(&mut self) -> anyhow::Result<&Vec<WaylandOutputInfo>> {
        self.poll_queue_until(|state| state.outputs_fetched)?;

        Ok(&self.state.outputs)
    }

    pub fn await_screenshot(&mut self) -> anyhow::Result<()> {
        self.poll_queue_until(|state| state.screenshot_ready)
    }

    pub fn create_shared_memory(&mut self) -> anyhow::Result<WaylandSharedMemory> {
        self.poll_queue_until(|state| state.current_frame.is_some())?;

        let (width, height, stride, format) = {
            let current_frame = self
                .state
                .current_frame
                .as_ref()
                .ok_or(WaylandError::BrokenState("current_frame"))?;

            let format = current_frame.format.ok_or(WaylandError::MissingFormat)?;

            let width = current_frame.width;
            let height = current_frame.height;
            let stride = current_frame.stride;

            (width, height, stride, format)
        };

        WaylandSharedMemory::new(
            self.state.wl_shm.as_ref().ok_or(WaylandError::NoShmBind)?,
            &self.queue.handle(),
            width,
            height,
            stride,
            format,
        )
    }

    fn poll_queue_until(
        &mut self,
        until: impl Fn(&WaylandScreenshotState) -> bool,
    ) -> anyhow::Result<()> {
        let start = SystemTime::now();

        while !until(&self.state) {
            self.queue.blocking_dispatch(&mut self.state)?;
            self.connection.flush()?;

            let diff = SystemTime::now().duration_since(start)?;

            if diff.as_secs() > 120 {
                return Err(WaylandError::EventQueueTimeout.into());
            }
        }

        Ok(())
    }

    pub fn next_screen(&mut self) {
        self.state.screenshot_ready = false;
        self.state.current_frame = None;
    }
}
