use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    screenshot_window::{ScreenshotWindowInit, ScreenshotWindowModel, ScreenshotWindowInput},
    ui::canvas::Canvas,
};
use crate::backend::{self, MonitorInfo};
use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub enum AppInput {}

#[derive(Debug)]
pub struct AppModel {
    window_senders: Vec<relm4::Sender<ScreenshotWindowInput>>,
    canvas: Rc<RefCell<Canvas>>,
}

impl SimpleComponent for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = ();
    type Root = gtk::Window;
    type Widgets = ();

    fn init_root() -> Self::Root {
        let window = gtk::Window::new();

        // we use this window only as a container for the screenshot windows
        window.hide();

        window
    }

    fn init(
        _payload: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut monitors = get_monitors();
        let (total_width, total_height) = get_total_view_size(&monitors.values().collect());

        let mut model = AppModel {
            window_senders: vec![],
            canvas: Rc::new(RefCell::new(
                Canvas::new(total_width, total_height).expect("Couldn't create the canvas."),
            )),
        };
        let app = relm4::main_application();
        register_keyboard_events(root);

        let sender_ref = Rc::new(sender);

        let screenshots = backend::wayland::create_screenshots().unwrap();

        for (output_info, image) in screenshots {
            if let MonitorInfo::Wayland { ref name, .. } = output_info.monitor_info {
                let monitor = monitors
                    .remove(&name.to_string())
                    .expect("We tried to access a non-existend monitor.");

                model
                    .canvas
                    .borrow_mut()
                    .stamp_image(
                        monitor.geometry().x() as f64,
                        monitor.geometry().y() as f64,
                        &image,
                    )
                    .expect("Couldn't stamp image.");

                let window = ScreenshotWindowModel::builder();
                app.add_window(&window.root);

                let mut window_connector = window.launch(ScreenshotWindowInit {
                    output_info: output_info.clone(),
                    monitor,
                    parent_sender: sender_ref.clone(),
                    canvas: model.canvas.clone(),
                });

                let window_sender = window_connector.sender().to_owned();
                model.window_senders.push(window_sender);

                window_connector.detach_runtime();
            }
        }

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {}
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

fn get_total_view_size(monitors: &Vec<&gdk4::Monitor>) -> (i32, i32) {
    let mut width = 0;
    let mut height = 0;
    for monitor in monitors {
        let m_width = monitor.geometry().width();
        let m_height = monitor.geometry().height();
        let x = monitor.geometry().x();
        let y = monitor.geometry().y();

        if width < x + m_width {
            width = x + m_width;
        }

        if height < y + m_height {
            height = y + m_height;
        }
    }

    (width, height)
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
