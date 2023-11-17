use thiserror::Error;

#[derive(Error, Debug)]
pub enum WaylandError {
    #[error("Compositor did not provide a shared memory binding")]
    NoShmBind,

    #[error("Compositor did not provide a screenshot manager")]
    NoScreenshotManager,

    #[error("Event Queue did not receive desired event")]
    EventQueueTimeout,

    #[error("We tried to access an uninitialized property. This is likely a bug")]
    BrokenState,

    #[error("The Wayland Compositor did not provide a format for the screenshot it made")]
    MissingFormat,

    #[error("We failed to convert the screenshot buffer into a image")]
    ConvertImageFailed,
}