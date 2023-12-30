use flakeshot::frontend::main_window::AppModel;
use relm4::{RelmApp, gtk::{CssProvider, self}};

fn main() {
    let app = RelmApp::new("org.flakeshot.app");
    relm4_icons::initialize_icons();
    initialize_css();

    app.run::<AppModel>(());
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
