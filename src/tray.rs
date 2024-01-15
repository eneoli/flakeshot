use ksni;
use tracing::Level;

#[derive(Debug)]
struct Tray;

impl ksni::Tray for Tray {
    fn icon_name(&self) -> String {
        "flakeshot-tray".into()
    }

    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        println!("Leftclick action is still under development.");
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![SubMenu {
            label: "System tray is under development".into(),
            ..Default::default()
        }
        .into()]
    }
}

pub fn start() {
    let span = tracing::span!(Level::TRACE, "tray");
    let _enter = span.enter();

    tracing::debug!("Starting tray");

    let _ = ksni::spawn(Tray).unwrap();

    loop {
        std::thread::park();
    }
}
