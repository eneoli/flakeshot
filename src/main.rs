#[cfg(not(target_family = "unix"))]
compile_error!("flakeshot only runs on UNIX-like systems.");

use clap::Parser;
use flakeshot::cli::Cli;
use flakeshot::frontend::window::mode::Mode;

fn main() {
    let cli = Cli::parse();

    flakeshot::init_logging(&cli.log_level, &cli.log_path);
    flakeshot::start(Mode::from(cli.command()));
}
