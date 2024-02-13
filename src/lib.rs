//! Welcome to the code-documentation of flakeshot!

use std::{fs::File, path::PathBuf, sync::OnceLock};

use clap::crate_name;
use cli::LogLevel;
use frontend::window::{main_window::AppModel, run_mode::RunMode};
use gtk4::{gdk::Display, CssProvider};
use relm4::RelmApp;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use xdg::BaseDirectories;

pub mod backend;
pub mod cli;
pub mod config;
pub mod frontend;
pub mod tray;

static XDG: OnceLock<BaseDirectories> = OnceLock::new();

pub const LOG_FILENAME: &str = "log.log";
pub const CONFIG_FILENAME: &str = "config.toml";

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

pub fn get_xdg() -> &'static BaseDirectories {
    XDG.get_or_init(|| BaseDirectories::with_prefix(crate_name!()).expect("Couldn't access XDG"))
}

fn get_default_log_path() -> PathBuf {
    get_xdg()
        .place_state_file(LOG_FILENAME)
        .unwrap_or_else(|e| panic!("Couldn't access log file path: {}", e))
}

fn get_default_config_path() -> PathBuf {
    get_xdg()
        .place_config_file(CONFIG_FILENAME)
        .unwrap_or_else(|e| panic!("Couldn't access config file path: {}", e))
}

pub fn start(mode: RunMode) {
    let app = RelmApp::new("org.flakeshot.app")
        .with_args(vec![])
        .visible_on_activate(false);
    relm4_icons::initialize_icons();
    initialize_css();

    app.run::<AppModel>(mode);
}

fn initialize_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("frontend/style.css"));

    gtk4::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
