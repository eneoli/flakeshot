use clap::Parser;
use flakeshot::cli::Command;
use flakeshot::{cli::Cli, daemon, tray};

fn main() {
    {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            tracing_panic::panic_hook(panic_info);
            prev_hook(panic_info);
        }));
    };

    flakeshot::init_xdg();

    let cli = Cli::parse();
    flakeshot::init_logging(&cli.log_level, &cli.log_path);
    flakeshot::init_socket_path();

    match cli.command() {
        Command::Gui => flakeshot::start_gui(),
        Command::Tray => tray::start(),
        Command::Daemon => daemon::start().expect("An error occured while running the daemon."),
    };
}
