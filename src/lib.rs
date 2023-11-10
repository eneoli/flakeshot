pub mod backend;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error occured in the backend: {0}")]
    Backend(#[from] backend::Error),
}
