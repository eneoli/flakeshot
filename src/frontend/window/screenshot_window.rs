use std::rc::Rc;

use gdk4_x11::X11Surface;
use gtk::{
    cairo::ImageSurface,
    glib::object::Cast,
    prelude::{EventControllerExt, MonitorExt, NativeExt},
};
use gtk4_layer_shell::LayerShell;
use relm4::{
    drawing::DrawHandler,
    gtk::{
        self,
        prelude::{GtkWindowExt, WidgetExt},
    },
    Component, ComponentController, ComponentParts, ComponentSender, Controller, Sender,
    SimpleComponent,
};
use x11rb::{
    connection::Connection,
    protocol::xproto::{ConfigureWindowAux, ConnectionExt},
};

use crate::{
    backend::is_wayland,
    frontend::{
        shape::point::Point,
        ui::toolbar::{Toolbar, ToolbarEvent},
    },
};

use super::main_window::AppModel;

pub struct ScreenshotWindowInit {
    pub monitor: gtk4::gdk::Monitor,
    pub parent_sender: Rc<relm4::ComponentSender<AppModel>>,
}

pub struct ScreenshotWindowModel {
    monitor: gtk4::gdk::Monitor,
    draw_handler: DrawHandler,
    surface: Option<ImageSurface>,
    toolbar: Controller<Toolbar>,
}

#[derive(Debug)]
pub enum MouseEvent {
    MouseMove(Point),
    MosePress { button: i32, position: Point },
    MouseRelease { button: i32, position: Point },
}

#[derive(Debug)]
pub enum ScreenshotWindowInput {
    Draw(ImageSurface),
    Redraw,
    EnterWindow,
    LeaveWindow,
    ToolbarEvent(ToolbarEvent),
    MouseEvent(MouseEvent),
}

#[derive(Debug)]
pub enum ScreenshotWindowOutput {
    ToolbarEvent(ToolbarEvent),
    MouseEvent(MouseEvent),
}

impl ScreenshotWindowModel {
    fn init(payload: ScreenshotWindowInit, input_sender: &Sender<ScreenshotWindowInput>) -> Self {
        let toolbar = Toolbar::builder()
            .launch(())
            .forward(input_sender, |event| {
                ScreenshotWindowInput::ToolbarEvent(event)
            });

        ScreenshotWindowModel {
            monitor: payload.monitor,
            draw_handler: DrawHandler::new(),
            toolbar,
            surface: None,
        }
    }

    fn draw(&mut self, surface: Option<ImageSurface>) {
        let ctx = self.draw_handler.get_context();

        if let Some(surface) = surface {
            self.surface = Some(surface);
        }

        if let Some(surface) = &self.surface {
            ctx.set_source_surface(surface, 0.0, 0.0)
                .expect("Couldn't set source surface.");

            ctx.paint().expect("Couldn't paint.");
        }
    }
}

impl SimpleComponent for ScreenshotWindowModel {
    type Input = ScreenshotWindowInput;
    type Output = ScreenshotWindowOutput;
    type Init = ScreenshotWindowInit;
    type Root = gtk::Window;
    type Widgets = ();

    fn init_root() -> Self::Root {
        let window = gtk4::Window::new();

        if is_wayland() {
            window.init_layer_shell();
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            window.set_layer(gtk4_layer_shell::Layer::Overlay);
            window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
        }

        window.set_decorated(false);
        window.add_css_class("screenshot_window");

        window
    }

    fn init(
        payload: ScreenshotWindowInit,
        window: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let mut model = ScreenshotWindowModel::init(payload, sender.input_sender());

        let width = model.monitor.geometry().width();
        let height = model.monitor.geometry().height();
        let monitor_x = model.monitor.geometry().x() as f64;
        let monitor_y = model.monitor.geometry().y() as f64;
        let realize_sender = sender.clone();

        window.set_visible(false); // unrealize window to prevent wayland protocol error when resizing
        window.set_default_size(width, height);

        if is_wayland() {
            window.set_monitor(&model.monitor);

            window.connect_realize(move |_| {
                // make sure window is finished rendering before first draw
                let s = realize_sender.clone();
                gtk::glib::idle_add_local_once(move || {
                    s.input(ScreenshotWindowInput::Redraw);
                });
            });
        } else {
            let (conn, _) = x11rb::connect(None).unwrap();
            let x11_window_config = ConfigureWindowAux::default()
                .x(monitor_x as i32)
                .y(monitor_y as i32);

            // move X11 Surface to right monitor on realize
            window.connect_realize(move |window| {
                if let Ok(desktop_session) = std::env::var("DESKTOP_SESSION") {
                    if ["gnome", "kde"].contains(&desktop_session.as_ref() as &&str) {
                        unimplemented!(concat![
                            "Flakeshot isn't working on gnome and kde at the moment, see:\n",
                            "https://github.com/eneoli/flakeshot/issues/91"
                        ]);
                    }
                }

                let surface = window.surface().downcast::<X11Surface>().unwrap();
                let xid = surface.xid();
                conn.configure_window(xid as u32, &x11_window_config)
                    .unwrap();

                conn.flush().unwrap();

                // make sure window is finished rendering before first draw
                let s = realize_sender.clone();
                gtk::glib::idle_add_local_once(move || {
                    s.input(ScreenshotWindowInput::Redraw);
                });
            });
        }

        window.set_visible(true);

        // those functions have to be called *after* `window.set_visible`
        if !is_wayland() {
            window.fullscreen_on_monitor(&model.monitor);
            window.fullscreen();
        }

        // Overlay
        let overlay = gtk::Overlay::new();
        window.set_child(Some(&overlay));

        // DrawingArea
        let drawing_area = model.draw_handler.drawing_area();
        drawing_area.set_size_request(width, height);
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);
        overlay.add_overlay(drawing_area);

        // Toolbar
        model.toolbar.widget().hide();
        model.toolbar.detach_runtime();
        overlay.add_overlay(model.toolbar.widget());

        // On Mouse Move/Enter/Leave
        let motion = gtk::EventControllerMotion::new();
        let motion_sender_enter = sender.clone();
        let motion_sender_leave = sender.clone();
        let motion_sender_move = sender.clone();

        motion.connect_enter(move |_, _, _| {
            motion_sender_enter.input(ScreenshotWindowInput::EnterWindow);
        });

        motion.connect_leave(move |_| {
            motion_sender_leave.input(ScreenshotWindowInput::LeaveWindow);
        });

        motion.connect_motion(move |_, x, y| {
            motion_sender_move.input(ScreenshotWindowInput::MouseEvent(MouseEvent::MouseMove(
                Point {
                    x: monitor_x + x,
                    y: monitor_y + y,
                },
            )));
        });

        overlay.add_controller(motion);

        // On Mouse Press/Release
        let gesture = gtk::GestureClick::new();

        let gesture_sender_pressed = sender.clone();
        gesture.connect_pressed(move |_, i, x, y| {
            gesture_sender_pressed.input(ScreenshotWindowInput::MouseEvent(
                MouseEvent::MosePress {
                    button: i,
                    position: Point {
                        x: monitor_x + x,
                        y: monitor_y + y,
                    },
                },
            ));
        });

        let gesture_sender_released = sender.clone();
        gesture.connect_released(move |_, i, x, y| {
            gesture_sender_released.input(ScreenshotWindowInput::MouseEvent(
                MouseEvent::MouseRelease {
                    button: i,
                    position: Point {
                        x: monitor_x + x,
                        y: monitor_y + y,
                    },
                },
            ));
        });

        gesture.set_propagation_phase(gtk::PropagationPhase::Bubble);

        drawing_area.add_controller(gesture);

        window.present();

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ScreenshotWindowInput::Draw(surface) => self.draw(Some(surface)),
            ScreenshotWindowInput::Redraw => self.draw(None),
            ScreenshotWindowInput::LeaveWindow => self.toolbar.widget().hide(),
            ScreenshotWindowInput::EnterWindow => self.toolbar.widget().show(),
            ScreenshotWindowInput::MouseEvent(event) => sender
                .output_sender()
                .emit(ScreenshotWindowOutput::MouseEvent(event)),
            ScreenshotWindowInput::ToolbarEvent(event) => sender
                .output_sender()
                .emit(ScreenshotWindowOutput::ToolbarEvent(event)),
        }
    }
}
