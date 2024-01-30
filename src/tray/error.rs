#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Couldn't acquire the socket: {0}")]
    AcquireSocket(rustix::io::Errno),

    #[error("The daemon isn't running yet. Please start it. (See help page.)")]
    NotRunning,

    #[error("There's already a daemon running.")]
    AlreadyRunning,
}
