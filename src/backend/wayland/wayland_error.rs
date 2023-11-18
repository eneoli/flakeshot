///
/// A general enum with possible errors as values which can occur in the wayland backend.
///
#[derive(thiserror::Error, Debug)]
pub enum WaylandError {
    #[error("We tried to access an uninitialized property: {0}. This is likely a bug")]
    BrokenState(&'static str),

    #[error("We failed to convert the screenshot buffer into an image")]
    ConvertImageFailed,

    #[error("There was an error while polling the event queue")]
    EventQueuePollingError(#[from] wayland_client::DispatchError),

    #[error("Event Queue did not receive desired event")]
    EventQueueTimeout,

    #[error("Compositor did not provide a shared memory binding")]
    NoShmBind,

    #[error("Compositor did not provide a screenshot manager")]
    NoScreenshotManager,

    #[error("The Wayland Compositor did not provide a format for the screenshot it made")]
    MissingFormat,

    #[error("Failed to spawn a thread")]
    ThreadSpawnFailed(#[from] tokio::task::JoinError),
}
