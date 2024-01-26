use clap::crate_name;
use ksni;
use notify_rust::Notification;

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
        kekw();
        // match daemon::acquire_lock() {
        //     Ok(Some(_)) => Notification::new()
        //         .appname(crate_name!())
        //         .summary(&format!("{}", daemon::Error::NotRunning))
        //         .show(),
        //     Err(e) => Notification::new()
        //         .appname(crate_name!())
        //         .summary(&format!("Couldn't test, if the daemon is running: {}", e))
        //         .show(),
        //     // Ok(None) => match daemon::send_message(Message::CreateScreenshot) {
        //     //     Ok(_) => Notification::new()
        //     //         .appname(crate_name!())
        //     //         .summary("Successfully created screenshot")
        //     //         .show(),
        //     //     Err(e) => Notification::new()
        //     //         .appname(crate_name!())
        //     //         .summary("Couldn't create screenshot")
        //     //         .body(&format!("{}", e))
        //     //         .show(),
        //     // },
        //     Ok(None) => Notification::new()
        //         .appname(crate_name!())
        //         .summary("Penis")
        //         .show(),
        // }
        // .unwrap();
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

fn kekw() {
    Notification::new()
        .appname(crate_name!())
        .summary("I use arch btw.")
        .show()
        .unwrap();
}
