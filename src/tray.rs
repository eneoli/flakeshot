use ksni;

use crate::daemon::message::Message;

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
        crate::daemon::send_message(Message::CreateScreenshot)
            .unwrap_or_else(|_| todo!("Better error handling"));
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![StandardItem {
            label: "Quit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into()]
    }
}

pub fn start() -> ! {
    tracing::debug!("Starting tray");

    ksni::spawn(Tray).expect("Couldn't spawn tray.");

    loop {
        std::thread::park();
    }
}
