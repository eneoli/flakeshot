#[cfg(not(target_family = "unix"))]
compile_error!("flakeshot only runs on UNIX-like systems.");

use clap::Parser;
use flakeshot::cli::Cli;
use flakeshot::frontend::window::run_mode::RunMode;
use notify_rust::Notification;

fn main() {
    trace_panics();
    let cli = Cli::parse();

    flakeshot::init_logging(&cli.log_level, &cli.log_path);
    flakeshot::start(RunMode::from(cli.command()));
}

fn trace_panics() {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        tracing_panic::panic_hook(panic_info);
        prev_hook(panic_info);

        // we are already in a panic hook, if that panics as well... then we are
        // entering a recursion loop so we're just ignoring the result
        let _ = Notification::new()
            .appname(clap::crate_name!())
            .urgency(notify_rust::Urgency::Critical)
            .summary("Flakeshot paniced!")
            .body(concat![
                "You likely found a bug. ",
                "Please take a look into the log file (see `flakeshot -h`) and ",
                "create an issue if suitable."
            ])
            .show();
    }));
}
