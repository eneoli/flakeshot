pub mod backend;

/// An enum error which contains all possible error sources while executing flakeshot.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error occured in the backend: {0}")]
    Backend(#[from] backend::Error),
}
