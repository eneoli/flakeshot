//! Welcome to the code-documentation of flakeshot!

use std::{fs::File, path::PathBuf, sync::OnceLock};

use clap::crate_name;
use cli::LogLevel;
use frontend::main_window::AppModel;
use gtk4::CssProvider;
use relm4::RelmApp;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use xdg::BaseDirectories;

pub mod backend;
pub mod cli;
pub mod daemon;
pub mod frontend;
pub mod tray;

pub static XDG: OnceLock<BaseDirectories> = OnceLock::new();
pub static SOCKET_PATH: OnceLock<PathBuf> = OnceLock::new();

const LOG_FILE: &str = "log.log";

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
    let log_file = File::create(path).expect("Couldn't create and open log path");

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

pub fn init_xdg() {
    let xdg = xdg::BaseDirectories::with_prefix(crate_name!()).expect("Couldn't access XDG");
    XDG.set(xdg).unwrap();
}

pub fn init_socket_path() {
    let socket_name = format!("{}.sock", crate_name!());
    let socket_file_path = XDG
        .get()
        .unwrap()
        .place_runtime_file(socket_name)
        .expect("Couldn't create socket file path.");

    SOCKET_PATH
        .set(socket_file_path)
        .expect("Couldn't set the socket file path in the code.");
}

pub fn get_default_log_path() -> PathBuf {
    XDG.get()
        .unwrap()
        .place_state_file(LOG_FILE)
        .unwrap_or_else(|e| panic!("Couldn't access log file path: {}", e))
}
