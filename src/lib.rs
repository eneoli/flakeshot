//! Welcome to the code-documentation of flakeshot!

use std::{cell::OnceCell, fs::File, path::PathBuf};

use clap::crate_name;
use cli::LogLevel;
use frontend::main_window::AppModel;
use gtk4::CssProvider;
use relm4::RelmApp;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_unwrap::ResultExt;

pub mod backend;
pub mod cli;
pub mod daemon;
pub mod frontend;
pub mod tray;

pub const SOCKET_PATH: OnceCell<PathBuf> = OnceCell::new();

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

pub fn init_logging(level: &LogLevel, path: &PathBuf) {
    let log_file =
        File::create(path).unwrap_or_else(|e| panic!("Couldn't create and open log path: {e}",));

    let subscriber_builder = tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_max_level(LevelFilter::from(level))
        .without_time()
        .with_ansi(true)
        .with_target(false)
        .with_file(true);

    if std::env::var_os("RUST_LOG").is_some() {
        subscriber_builder
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        subscriber_builder.init();
    }

    tracing::debug!("Logger initialised");
}

pub fn start_gui() {
    let app = RelmApp::new("org.flakeshot.app");
    relm4_icons::initialize_icons();
    initialize_css();

    app.run::<AppModel>(());
}

fn initialize_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("frontend/style.css"));

    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn init_socket_path() {
    let socket_name = format!("{}.sock", crate_name!());
    let xdg = xdg::BaseDirectories::new().expect_or_log("Couldn't access XDG.");
    let socket_file_path = xdg
        .place_runtime_file(socket_name)
        .expect_or_log("Couldn't create socket file path.");

    SOCKET_PATH
        .set(socket_file_path)
        .expect_or_log("Couldn't set the socket file path in the code.");
}
