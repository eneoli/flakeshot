use std::{collections::HashMap, rc::Rc};

use super::{
    screenshot_window::{
        ScreenshotWindowInit, ScreenshotWindowInput, ScreenshotWindowModel, ScreenshotWindowOutput,
    },
    ui_manager::UiManager,
};
use crate::backend::{self, MonitorInfo, OutputInfo};
use cairo::glib::Bytes;
use gtk::prelude::*;
use image::{DynamicImage, RgbaImage};
use relm4::{gtk::Application, prelude::*, Sender};

#[derive(Debug)]
pub enum AppInput {
    ScreenshotWindowOutput(ScreenshotWindowOutput),
}

pub struct AppModel {
    ui_manager: UiManager,
    window_senders: Vec<Sender<ScreenshotWindowInput>>,
}

impl AppModel {
    fn init(total_width: i32, total_height: i32) -> Self {
        AppModel {
            ui_manager: UiManager::new(total_width, total_height),
            window_senders: vec![],
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

        let mut model = Self::init(total_width, total_height);

        let screenshots =
            backend::create_screenshots().expect("We couldn't create the initial screenshots.");
        for screenshot in screenshots {
            init_monitor(&app, &mut model, &sender_ref, &screenshot, &mut monitors);
        }

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInput::ScreenshotWindowOutput(ScreenshotWindowOutput::ToolbarEvent(event)) => {
                self.ui_manager.handle_tool_event(event)
            }
            AppInput::ScreenshotWindowOutput(ScreenshotWindowOutput::MouseEvent(event)) => {
                self.ui_manager.handle_mouse_event(event)
            }
        }
    }
}

fn init_monitor(
    app: &Application,
    model: &mut AppModel,
    sender_ref: &Rc<ComponentSender<AppModel>>,
    (output_info, image): &(OutputInfo, DynamicImage),
    monitors: &mut HashMap<String, gdk4::Monitor>,
) {
    let (monitor, x, y, width, height) = {
        let monitor_name = match &output_info.monitor_info {
            MonitorInfo::Wayland { name, .. } => name,
            MonitorInfo::X11 { name } => name,
        };

        let monitor = monitors
            .remove(&monitor_name.to_string())
            .expect("We tried to access a non-existend monitor.");

        let x = monitor.geometry().x();
        let y = monitor.geometry().y();
        let width = monitor.geometry().width();
        let height = monitor.geometry().height();

        (monitor, x, y, width, height)
    };

    let window = ScreenshotWindowModel::builder();
    register_keyboard_events(&window.root);
    app.add_window(&window.root);

    // launch + forward messages to main window
    let mut window_controller = window
        .launch(ScreenshotWindowInit {
            monitor,
            parent_sender: sender_ref.clone(),
        })
        .forward(sender_ref.input_sender(), |event| {
            AppInput::ScreenshotWindowOutput(event)
        });

    model
        .window_senders
        .push(window_controller.sender().clone());

    window_controller.detach_runtime();

    // subscribe to canvas changes
    let sender_ui = window_controller.sender().clone();
    model.ui_manager.on_render(move |ui_manager| {
        let surface = ui_manager
            .crop(x as f64, y as f64, width, height)
            .expect("Couldn't crop surface for monitor.");
        sender_ui
            .send(ScreenshotWindowInput::Draw(surface))
            .expect("Letting window redraw canvas failed.");
    });

    // let resized_image = {
        // let img = image::imageops::resize(
            // image,
            // width as u32,
            // height as u32,
            // image::imageops::FilterType::Triangle,
        // );

        // DynamicImage::ImageRgba8(img)
    // };

    // add screenshot of monitor to image
    model
        .ui_manager
        .stamp_image(x as f64, y as f64, width as f64, height as f64, &image)
        .expect("Couldn't stamp image.");
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
        if let gdk4::Key::Escape = key {
            std::process::exit(0);
        }

        gtk::glib::Propagation::Proceed
    });

    window.add_controller(event_controller);
}
