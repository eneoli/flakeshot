use clap::Parser;
use flakeshot::cli::Command;
use flakeshot::{cli::Cli, daemon, tray};

fn main() {
    let cli = Cli::parse();

    flakeshot::init_logging(&cli.log_level, &cli.log_path);
    flakeshot::init_socket_path();

    match cli.command() {
        Command::Gui => flakeshot::start_gui(),
        Command::Tray => tray::start(),
        Command::Daemon => daemon::start(),
    };
}
