use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{
    screenshot_window::{ScreenshotWindowInit, ScreenshotWindowModel, ScreenshotWindowOutput},
    ui::{canvas::Canvas, toolbar::ToolbarEvent},
};
use crate::{
    backend::{self, MonitorInfo},
    frontend::file_chooser::FileChooser,
};
use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Debug)]
pub enum AppInput {
    ScreenshotWindowOutput(ScreenshotWindowOutput),
}

#[derive(Debug)]
pub struct AppModel {
    canvas: Rc<RefCell<Canvas>>,
}

impl AppModel {
    fn init(total_width: i32, total_height: i32) -> Self {
        AppModel {
            canvas: Rc::new(RefCell::new(
                Canvas::new(total_width, total_height).expect("Couldn't create the canvas."),
            )),
        }
    }
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
        _root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let app = relm4::main_application();
        let sender_ref = Rc::new(sender);

        let mut monitors = get_monitors();
        let (total_width, total_height) = get_total_view_size(&monitors.values().collect());

        let model = Self::init(total_width, total_height);

        let screenshots =
            backend::create_screenshots().expect("We couldn't create the initial screenshots.");
        for (output_info, image) in screenshots {
            let monitor_name = match output_info.monitor_info {
                MonitorInfo::Wayland { name, .. } => name,
                MonitorInfo::X11 { .. } => todo!(), // TODO #58
            };

            let monitor = monitors
                .remove(&monitor_name.to_string())
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
            register_keyboard_events(&window.root);

            window
                .launch(ScreenshotWindowInit {
                    monitor,
                    parent_sender: sender_ref.clone(),
                    canvas: model.canvas.clone(),
                })
                .forward(&(sender_ref.input_sender()), |event| {
                    AppInput::ScreenshotWindowOutput(event)
                })
                .detach_runtime();
        }

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::ScreenshotWindowOutput(ScreenshotWindowOutput::ToolbarEvent(event)) => {
                match event {
                    ToolbarEvent::SaveAsFile => {
                        let canvas_ref = self.canvas.clone();

                        FileChooser::open(move |file| {
                            if let Some(path) = file {
                                let width = canvas_ref.borrow().width() as u32;
                                let height = canvas_ref.borrow().height() as u32;

                                canvas_ref
                                    .borrow()
                                    .crop_to_image(0.0, 0.0, width, height)
                                    .expect("Couldn't crop canvas")
                                    .save(path)
                                    .expect("Couldn't save image.");
                            }
                        });
                    }
                    ToolbarEvent::SaveIntoClipboard => {}
                }
            }
        }
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
