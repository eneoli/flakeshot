use thiserror::Error;

#[derive(Error, Debug)]
pub enum WaylandError {
    #[error("Compositor did not provide a shared memory binding")]
    NoShmBind,

    #[error("Compositor did not provide a screenshot manager")]
    NoScreenshotManager,
}