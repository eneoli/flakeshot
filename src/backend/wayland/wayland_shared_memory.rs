use std::fs;
use std::fs::File;
use std::os::fd::AsFd;
use std::time::SystemTime;
use wayland_client::protocol::wl_buffer::WlBuffer;
use wayland_client::protocol::wl_shm::{Format, WlShm};
use wayland_client::protocol::wl_shm_pool::WlShmPool;
use wayland_client::{QueueHandle};
use crate::backend::wayland::wayland_screenshot_state::WaylandScreenshotState;

///
/// Wrapper around wayland shared memory.
/// Handles shared memory a bit more comfortable.
///
pub struct WaylandSharedMemory {
    memfile: File, // File handle for the shared memory
    shm_pool: WlShmPool, // Wayland memory pool mapped to the shared memory
    buffer: WlBuffer, // Buffer using the memory buffer
}

impl WaylandSharedMemory {
    pub fn new(
        wl_shm: &WlShm,
        queue_handle: &QueueHandle<WaylandScreenshotState>,
        width: u32,
        height: u32,
        stride: u32,
        format: Format,
    ) -> anyhow::Result<WaylandSharedMemory> {
        let memfile = create_shm_file("flakeshot_pool", (height * stride) as u64)?;

        let shm_pool = wl_shm.create_pool(
            memfile.as_fd(),
            (height * stride) as i32,
            &queue_handle,
            (),
        );

        let buffer = shm_pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            format,
            &queue_handle,
            (),
        );

        Ok(
            Self {
                memfile,
                shm_pool,
                buffer,
            }
        )
    }

    pub fn destroy(self: &mut Self) {
        self.shm_pool.destroy();
        self.buffer.destroy();
    }

    pub fn get_buffer(self: &Self) -> &WlBuffer {
        &self.buffer
    }

    pub fn get_memfile(self: &Self) -> &File {
        &self.memfile
    }
}

fn create_shm_file(prefix: &str, bytes: u64) -> anyhow::Result<File> {
    let name = gen_random_file_name(prefix)?;

    let options = memfd::MemfdOptions::default()
        .allow_sealing(true);

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

    while file_exists(&file_name.as_str()) {
        duration = duration + 1;
    }

    Ok(file_name)
}

fn file_exists(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}