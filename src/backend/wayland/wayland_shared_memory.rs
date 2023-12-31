use crate::backend::wayland::wayland_screenshot_state::WaylandScreenshotState;
use std::fs;
use std::fs::File;
use std::os::fd::AsFd;
use std::time::SystemTime;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_shm::{Format, WlShm};
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::QueueHandle;

use super::wayland_error::WaylandError;

///
/// Wrapper around wayland shared memory.
/// Handles shared memory a bit more comfortable.
///
pub struct WaylandSharedMemory {
    width: u32,
    height: u32,
    format: Format,
    memfile: File,
    shm_pool: WlShmPool,
    buffer: WlBuffer,
}

impl WaylandSharedMemory {
    pub fn new(
        wl_shm: &WlShm,
        queue_handle: &QueueHandle<WaylandScreenshotState>,
        width: u32,
        height: u32,
        stride: u32,
        format: Format,
    ) -> Result<WaylandSharedMemory, WaylandError> {
        let memfile = create_shm_file("flakeshot_pool", (height * stride) as u64)
            .map_err(|_| WaylandError::ShmCreationFailed)?;

        let shm_pool =
            wl_shm.create_pool(memfile.as_fd(), (height * stride) as i32, queue_handle, ());

        let buffer = shm_pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            format,
            queue_handle,
            (),
        );

        Ok(Self {
            width,
            height,
            format,
            memfile,
            shm_pool,
            buffer,
        })
    }

    pub fn destroy(&mut self) {
        self.shm_pool.destroy();
        self.buffer.destroy();
    }

    pub fn get_buffer(&self) -> &WlBuffer {
        &self.buffer
    }

    pub fn get_memfile(&self) -> &File {
        &self.memfile
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn format(&self) -> Format {
        self.format
    }
}

fn create_shm_file(prefix: &str, bytes: u64) -> anyhow::Result<File> {
    let name = gen_random_file_name(prefix)?;

    let options = memfd::MemfdOptions::default().allow_sealing(true);

    let memfile = options.create(name)?;

    memfile.as_file().set_len(bytes)?;

    memfile.add_seals(&[
        memfd::FileSeal::SealShrink,
        memfd::FileSeal::SealGrow,
        memfd::FileSeal::SealSeal,
    ])?;

    Ok(memfile.into_file())
}

fn gen_random_file_name(prefix: &str) -> anyhow::Result<String> {
    let mut duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis();

    let file_name = prefix.to_string() + duration.to_string().as_str();

    while file_exists(file_name.as_str()) {
        duration += 1;
    }

    Ok(file_name)
}

fn file_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}
