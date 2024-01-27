use ksni;
use tracing::{error, info, warn};

use crate::daemon::{self, message::Message};

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
        match daemon::acquire_lock() {
            Ok(Some(_)) => warn!("{}", daemon::Error::NotRunning),
            Err(e) => error!("Couldn't test, if the daemon is running: {}", e),
            Ok(None) => match daemon::send_message(Message::CreateScreenshot) {
                Ok(_) => info!("Screenshot successfully created."),
                Err(e) => error!("Couldn't create screenshot: {}", e),
            },
        };
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
