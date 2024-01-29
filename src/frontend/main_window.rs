use std::{collections::HashMap, rc::Rc};

use super::{
    screenshot_window::{
        ScreenshotWindowInit, ScreenshotWindowInput, ScreenshotWindowModel, ScreenshotWindowOutput,
    },
    ui_manager::UiManager,
};
use crate::{
    backend::{self, MonitorInfo, OutputInfo},
    daemon::message::Message,
    get_socket_file_path,
};

use anyhow::Context;
use gtk::prelude::*;
use image::DynamicImage;
use relm4::{prelude::*, Sender};
use tokio::{io::Interest, net::UnixListener};
use tracing::{debug, error};

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

impl Component for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = ();
    type Root = gtk::Window;
    type Widgets = ();
    type CommandOutput = Message;

    fn init_root() -> Self::Root {
        let window = gtk::Window::new();

        // we use this window only as a container for the screenshot windows
        window.set_visible(false);

        window
    }

    fn init(
        _payload: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        // start listenting on the socket
        sender.command(|out, shutdown| shutdown.register(listen_to_socket(out)).drop_on_shutdown());

        let model = {
            let monitors = get_monitors();
            let (total_width, total_height) = get_total_view_size(&monitors.values().collect());

            Self::init(total_width, total_height)
        };

        ComponentParts { model, widgets: () }
    }

    fn update_cmd(&mut self, message: Message, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Message::CreateScreenshot => self.start_gui(sender),
        }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AppInput::ScreenshotWindowOutput(ScreenshotWindowOutput::ToolbarEvent(event)) => {
                self.ui_manager.handle_tool_event(event)
            }
        }
    }
}

impl AppModel {
    fn start_gui(&mut self, sender: ComponentSender<Self>) {
        let sender = Rc::new(sender);
        let screenshots =
            backend::create_screenshots().expect("We couldn't create the initial screenshots.");
        let mut monitors = get_monitors();

        for screenshot in screenshots {
            self.init_monitor(sender.clone(), &screenshot, &mut monitors);
        }
    }

    fn init_monitor(
        &mut self,
        sender: Rc<ComponentSender<Self>>,
        (output_info, image): &(OutputInfo, DynamicImage),
        monitors: &mut HashMap<String, gdk4::Monitor>,
    ) {
        let app = relm4::main_application();

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
                parent_sender: sender.clone(),
            })
            .forward(sender.input_sender(), |event| {
                AppInput::ScreenshotWindowOutput(event)
            });
        window_controller.detach_runtime();

        self.window_senders.push(window_controller.sender().clone());

        // subscribe to canvas changes
        let sender_ui = window_controller.sender().clone();
        self.ui_manager.on_render(move |ui_manager| {
            let surface = ui_manager
                .crop(x as f64, y as f64, width, height)
                .expect("Couldn't crop surface for monitor.");
            sender_ui
                .send(ScreenshotWindowInput::Draw(surface))
                .expect("Letting window redraw canvas failed.");
        });

        // add screenshot of monitor to image
        self.ui_manager
            .stamp_image(x as f64, y as f64, image)
            .expect("Couldn't stamp image.");
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
    let window2 = window.clone();

    event_controller.connect_key_pressed(move |_, key, _, _| {
        if let gdk4::Key::Escape = key {
            window2.set_visible(false);
        }

        gtk::glib::Propagation::Proceed
    });

    window.add_controller(event_controller);
}

async fn listen_to_socket(out: Sender<Message>) {
    let listener = {
        let socket_path = get_socket_file_path();
        UnixListener::bind(socket_path)
            .context("Couldn't bind to socket.")
            .unwrap()
    };
    debug!("Socket listener created");

    let mut byte_buffer: Vec<u8> = vec![];
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                if let Err(e) = stream.ready(Interest::READABLE).await {
                    error!(
                        "An IO error occured while waiting for messages of the listener: {}",
                        e
                    );
                }

                match stream.try_read_buf(&mut byte_buffer) {
                    Ok(amount_bytes) if amount_bytes > 0 => process_message(&mut byte_buffer, &out),
                    Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => {
                        error!(
                            "An error occured while trying to read the message from the socket: {}",
                            e
                        );
                    }
                    _ => {}
                };
            }
            Err(e) => error!("Coulnd't connect to listener: {}", e),
        }
    }
}

fn process_message(buffer: &mut Vec<u8>, out: &Sender<Message>) {
    let msg: Message = {
        let bytes = std::mem::take(buffer);
        let string = String::from_utf8(bytes).unwrap();
        ron::from_str(&string).unwrap()
    };

    out.send(msg).unwrap();
}
