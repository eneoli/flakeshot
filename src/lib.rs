//! Welcome to the code-documentation of flakeshot!
pub mod backend;
pub mod cli;
pub mod tray;

/// An enum error which contains all possible error sources while executing flakeshot.
///
/// # Convention
/// Just click on the `Error` value of each error-enum-value to get more information about them.
///
/// ## Example
/// If you want to understand what [`Error::Backend`] catches, then just click
/// on its `Error` type and you should get to [`backend::Error`] where a more detailed
/// description waits for you!
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error occured in the backend: {0}")]
    Backend(#[from] backend::Error),
}
