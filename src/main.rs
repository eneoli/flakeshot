use flakeshot::frontend::main_window::AppModel;
use relm4::RelmApp;

fn main() {
    let app = RelmApp::new("org.flakeshot.app");

    app.run::<AppModel>(());
}
