//! Contains the Cli implementation of flakeshot.
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
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
