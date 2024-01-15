use clap::Parser;
use flakeshot::cli::Command;
use flakeshot::frontend::main_window::AppModel;
use flakeshot::{cli::Cli, tray};
use relm4::{
    gtk::{self, CssProvider},
    RelmApp,
};
use tracing::Level;

fn main() {
    let cli = Cli::parse();

    flakeshot::init_logging(&cli.log_level, &cli.log_path);

    match cli.command() {
        Command::Gui => start_gui(),
        Command::Tray => tray::start(),
    };
}

fn initialize_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("frontend/style.css"));

    gtk::style_context_add_provider_for_display(
        &gdk4::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn start_gui() {
    let span = tracing::span!(Level::TRACE, "gui");
    let _enter = span.enter();

    tracing::debug!("Starting gui");

    let app = RelmApp::new("org.flakeshot.app");
    relm4_icons::initialize_icons();
    initialize_css();

    app.run::<AppModel>(());
}
