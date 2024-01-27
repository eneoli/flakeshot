#[cfg(not(target_family = "unix"))]
compile_error!("flakeshot only runs on UNIX-like systems.");

use clap::Parser;
use flakeshot::cli::Command;
use flakeshot::{cli::Cli, tray};

fn main() {
    let cli = Cli::parse();

    match cli.command() {
        Command::Gui => flakeshot::start_gui(),
        Command::Tray => tray::start(),
    };
}
