use std::collections::HashMap;

use super::screenshot_window::{ScreenshotWindowInit, ScreenshotWindowModel};
use crate::backend::{self, MonitorInfo};
use gtk::prelude::*;
use relm4::prelude::*;

pub struct AppModel {}

impl SimpleComponent for AppModel {
    type Input = ();
    type Output = ();
    type Init = ();
    type Root = gtk::Window;
    type Widgets = ();

    fn init_root() -> Self::Root {
        let window = gtk::Window::new();

        window.hide(); // we use this window only as a container for the screenshot windows

        window
    }

    fn init(
        _payload: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = AppModel {};
        let app = relm4::main_application();
        register_keyboard_events(root);

        let screenshots = backend::wayland::create_screenshots().unwrap();
        let mut monitors = get_monitors();

        for (output_info, image) in screenshots {
            if let MonitorInfo::Wayland { ref name, .. } = output_info.monitor_info {
                let monitor = monitors
                    .remove(&name.to_string())
                    .expect("We tried to access a non-existend monitor.");

                let window = ScreenshotWindowModel::builder();
                app.add_window(&window.root);

                window
                    .launch(ScreenshotWindowInit {
                        output_info: output_info.clone(),
                        image: image.clone(),
                        monitor,
                    })
                    .detach_runtime();
            }
        }

        ComponentParts { model, widgets: () }
    }
}

fn get_monitors() -> HashMap<String, gdk4::Monitor> {
    let monitor_list_model = gdk4::Display::default()
        .expect("GDK did not provide a display for us.")
        .monitors();

    let mut monitors = HashMap::new();
    for i in 0..(monitor_list_model.n_items()) {
        let monitor = monitor_list_model
            .item(i)
            .expect("We tried to access an invalid monitor.")
            .downcast::<gdk4::Monitor>()
            .expect("Provided object is not a GDK Monitor");

        let connector = monitor
            .connector()
            .expect("GDK did not provide a monitor connector for us.");

        monitors.insert(connector.to_string(), monitor);
    }

    monitors
}

fn register_keyboard_events(window: &gtk::Window) {
    let event_controller = gtk::EventControllerKey::new();

    event_controller.connect_key_pressed(|_, key, _, _| {
        match key {
            gdk4::Key::Escape => {
                std::process::exit(0);
            }
            _ => (),
        }
        gtk::glib::Propagation::Proceed
    });

    window.add_controller(event_controller);
}
