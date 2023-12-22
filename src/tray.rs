use ksni;

#[derive(Debug)]
struct Tray;

impl ksni::Tray for Tray {
    fn icon_name(&self) -> String {
        "flakeshot-tray".into()
    }

    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
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
    let _ = ksni::spawn(Tray).unwrap();

    loop {
        std::thread::park();
    }
}
