#[cfg(not(target_family = "unix"))]
compile_error!("flakeshot only runs on UNIX-like systems.");

use clap::Parser;
use flakeshot::cli::Cli;
use flakeshot::frontend::window::mode::Mode;

fn main() {
    let cli = Cli::parse();

    flakeshot::start(Mode::from(cli.command()));
}
