#[cfg(not(target_family = "unix"))]
compile_error!("flakeshot only runs on UNIX-like systems.");

use clap::Parser;
use flakeshot::cli::Command;
use flakeshot::frontend::window::mode::Mode;
use flakeshot::{cli::Cli, tray};

fn main() {
    let cli = Cli::parse();

    flakeshot::start(Mode::from(cli.command()));

    // match cli.command() {
    //     Command::Gui => flakeshot::start(),
    //     Command::Tray => tray::start(),
    // };
}
