//! Contains the Cli implementation of flakeshot.
use std::{fmt::Display, path::PathBuf};

use clap::{crate_name, Parser, Subcommand, ValueEnum};
use tracing::level_filters::LevelFilter;

pub const LOG_FILE: &str = "log.log";

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short, long, default_value_t = LogLevel::Error,  help = "",
        long_help = concat!(
        "*Note*: You can also set the log level through the `RUST_LOG` environment variable and filter the logs.\n",
        "See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives\n",
        "\n",
        "Example: `RUST_LOG=[tray]=debug ", crate_name!(), " - This will enable logs only related to the tray in debug mode."
    ))]
    pub log_level: LogLevel,

    #[arg(long, default_value = get_default_log_path().into_os_string())]
    pub log_path: PathBuf,

    #[command(subcommand)]
    command: Option<Command>,
}

impl Cli {
    pub fn command(&self) -> Command {
        self.command.unwrap_or(Command::Tray)
    }
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum Command {
    /// Open the manual capture ui
    Gui,

    /// Start the system tray of flakeshot. (default)
    Tray,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Off,
}

impl From<&LogLevel> for LevelFilter {
    fn from(level: &LogLevel) -> Self {
        match level {
            LogLevel::Error => Self::ERROR,
            LogLevel::Warn => Self::WARN,
            LogLevel::Info => Self::INFO,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Trace => Self::TRACE,
            LogLevel::Off => Self::OFF,
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", LevelFilter::from(self))
    }
}

fn get_default_log_path() -> PathBuf {
    let xdg = xdg::BaseDirectories::new().unwrap();

    let mut log_file_path = xdg.create_state_directory(crate_name!()).unwrap();
    log_file_path.push(LOG_FILE);

    xdg.place_state_file(log_file_path).unwrap()
}
