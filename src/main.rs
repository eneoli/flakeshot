#[cfg(not(target_family = "unix"))]
compile_error!("flakeshot only runs on UNIX-like systems.");

use clap::Parser;
use flakeshot::cli::{Cli, Command};

fn main() -> anyhow::Result<()> {
    {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            tracing_panic::panic_hook(panic_info);
            prev_hook(panic_info);
        }));
    };

    let cli = Cli::parse();
    flakeshot::init_logging(&cli.log_level, &cli.log_path);

    match cli.command() {
        Command::Gui => flakeshot::start(),
        Command::Tray => flakeshot::frontend::start(),
    }
}
