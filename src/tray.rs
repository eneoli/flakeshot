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

    fn activate(&mut self, _x: i32, _y: i32) {
        crate::start_gui();
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
          StandardItem {
              label: "Quit".into(),
              activate: Box::new(|_| std::process::exit(0)),
              ..Default::default()
          }
          .into(),
        ]
    }
}

pub fn start() {
    let _ = ksni::spawn(Tray).unwrap();

    loop {
        std::thread::park();
    }
}
