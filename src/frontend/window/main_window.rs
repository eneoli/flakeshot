use std::{collections::HashMap, rc::Rc};

use super::{
    mode::Mode,
    screenshot_window::{
        ScreenshotWindowInit, ScreenshotWindowInput, ScreenshotWindowModel, ScreenshotWindowOutput,
    },
};
use crate::{
    backend::{self, MonitorInfo, OutputInfo},
    frontend::ui::ui_manager::UiManager,
    tray,
};
use gtk::prelude::*;
use image::DynamicImage;
use relm4::{gtk::Application, prelude::*};

#[derive(Debug)]
pub enum AppInput {
    ScreenshotWindowOutput(ScreenshotWindowOutput),
}

pub struct AppModel {
    mode: Mode,
    ui_manager: Option<UiManager>,
    window_controllers: Vec<Controller<ScreenshotWindowModel>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Close,
    Quit,
    Gui,
}

impl AppModel {
    fn init(mode: Mode) -> Self {
        AppModel {
            mode,
            ui_manager: None,
            window_controllers: vec![],
        }
    }

    fn start_gui(&mut self, sender: ComponentSender<Self>) {
        let sender_ref = Rc::new(sender);
        let mut monitors = get_monitors();

        let mut ui_manager = {
            let (total_width, total_height) = get_total_view_size(&monitors.values().collect());
            UiManager::new(total_width, total_height)
        };

        let screenshots =
            backend::create_screenshots().expect("We couldn't create the initial screenshots.");
        let app = relm4::main_application();
        for screenshot in screenshots {
            self.init_monitor(
                &app,
                &mut ui_manager,
                &sender_ref,
                &screenshot,
                &mut monitors,
            );
        }

        ui_manager.persist_canvas();

        self.ui_manager = Some(ui_manager);
    }

    fn init_monitor(
        &mut self,
        app: &Application,
        ui_manager: &mut UiManager,
        sender_ref: &Rc<ComponentSender<Self>>,
        (output_info, image): &(OutputInfo, DynamicImage),
        monitors: &mut HashMap<String, gtk4::gdk::Monitor>,
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

        let window = {
            let window = ScreenshotWindowModel::builder();
            register_keyboard_events(&window.root, sender_ref.clone());
            app.add_window(&window.root);
            window.root.set_visible(false);

            window
        };

        // launch + forward messages to main window
        let window_controller = {
            let window_controller = window
                .launch(ScreenshotWindowInit {
                    monitor,
                    parent_sender: sender_ref.clone(),
                })
                .forward(sender_ref.input_sender(), |event| {
                    AppInput::ScreenshotWindowOutput(event)
                });

            // subscribe to canvas changes
            let sender_ui = window_controller.sender().clone();
            ui_manager.on_render(move |ui_manager| {
                let surface = ui_manager
                    .crop(x as f64, y as f64, width, height)
                    .expect("Couldn't crop surface for monitor.");
                sender_ui
                    .send(ScreenshotWindowInput::Draw(surface))
                    .expect("Letting window redraw canvas failed.");
            });

            window_controller
        };

        self.window_controllers.push(window_controller);

        // add screenshot of monitor to image
        ui_manager
            .stamp_image(x as f64, y as f64, width as f64, height as f64, image)
            .expect("Couldn't stamp image.");
    }

    fn close(&mut self) {
        match self.mode {
            Mode::Tray => {
                self.ui_manager = None;
                for controller in &self.window_controllers {
                    controller.widget().close();
                }
            }
            Mode::Gui => self.quit(),
        };
    }

    fn quit(&mut self) {
        relm4::main_application().quit();
    }
}

impl Component for AppModel {
    type Input = AppInput;
    type Output = ();
    type Init = Mode;
    type Root = gtk::Window;
    type Widgets = ();
    type CommandOutput = Command;

    fn init_root() -> Self::Root {
        let window = gtk::Window::new();

        // we use this window only as a container for the screenshot windows
        window.minimize();
        window.set_visible(false);

        window
    }

    fn init(
        payload: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut model = Self::init(payload);

        if payload == Mode::Gui {
            model.start_gui(sender);
        } else {
            sender.command(|out, shutdown| shutdown.register(tray::start(out)).drop_on_shutdown());
        }

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        if let Some(ui_manager) = &mut self.ui_manager {
            match message {
                AppInput::ScreenshotWindowOutput(ScreenshotWindowOutput::ToolbarEvent(event)) => {
                    ui_manager.handle_tool_event(event)
                }
                AppInput::ScreenshotWindowOutput(ScreenshotWindowOutput::MouseEvent(event)) => {
                    ui_manager.handle_mouse_event(event)
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            Command::Quit => self.quit(),
            Command::Gui => self.start_gui(sender),
            Command::Close => self.close(),
        }
    }
}

fn get_monitors() -> HashMap<String, gtk4::gdk::Monitor> {
    let monitor_list_model = gtk4::gdk::Display::default()
        .expect("GDK did not provide a display for us.")
        .monitors();

    let mut monitors = HashMap::new();
    for i in 0..(monitor_list_model.n_items()) {
        let monitor = monitor_list_model
            .item(i)
            .expect("We tried to access an invalid monitor.")
            .downcast::<gtk4::gdk::Monitor>()
            .expect("Provided object is not a GDK Monitor");

        let connector = monitor
            .connector()
            .expect("GDK did not provide a monitor connector for us.");

        monitors.insert(connector.to_string(), monitor);
    }

    monitors
}

fn get_total_view_size(monitors: &Vec<&gtk4::gdk::Monitor>) -> (i32, i32) {
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

fn register_keyboard_events(window: &gtk::Window, sender: Rc<ComponentSender<AppModel>>) {
    let event_controller = gtk::EventControllerKey::new();

    event_controller.connect_key_pressed(move |_, key, _, _| {
        if let gtk4::gdk::Key::Escape = key {
            sender
                .command_sender()
                .send(Command::Close)
                .expect("Couldn't send quit command");
        }

        gtk::glib::Propagation::Proceed
    });

    window.add_controller(event_controller);
}
