use std::rc::Rc;

use cairo::ImageSurface;
use gdk4::prelude::MonitorExt;
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

use crate::backend::is_wayland;

use super::{
    main_window::AppModel,
    ui::toolbar::{Toolbar, ToolbarEvent},
};

pub struct ScreenshotWindowInit {
    pub monitor: gdk4::Monitor,
    pub parent_sender: Rc<relm4::ComponentSender<AppModel>>,
}

pub struct ScreenshotWindowModel {
    monitor: gdk4::Monitor,
    draw_handler: DrawHandler,
    surface: Option<ImageSurface>,
    toolbar: Controller<Toolbar>,
}

#[derive(Debug)]
pub enum ScreenshotWindowInput {
    Draw(ImageSurface),
    Redraw,
    EnterWindow,
    LeaveWindow,
    ToolbarEvent(ToolbarEvent),
}

#[derive(Debug)]
pub enum ScreenshotWindowOutput {
    ToolbarEvent(ToolbarEvent),
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
            ctx.set_source_surface(&surface, 0.0, 0.0)
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
        let window = gtk::Window::new();

        if is_wayland() {
            window.init_layer_shell();
            window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            window.set_layer(gtk4_layer_shell::Layer::Overlay);
            window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
        } else {
            window.fullscreen();
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
        let mut model = ScreenshotWindowModel::init(payload, &sender.input_sender());

        let width = model.monitor.geometry().width();
        let height = model.monitor.geometry().height();

        // Window size
        window.hide(); // unrealize window to prevent wayland protocol error when resizing
        window.set_monitor(&model.monitor);
        window.set_default_size(width, height);

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

        // On Realize
        let realize_sender = sender.clone();
        window.connect_realize(move |_| {
            let s = realize_sender.clone();

            // make sure window is finished rendering before first draw
            gtk::glib::idle_add_local_once(move || {
                s.input(ScreenshotWindowInput::Redraw);
            });
        });

        // On Mouse Enter/Leave
        let motion = gtk::EventControllerMotion::new();
        let motion_sender_enter = sender.clone();
        let motion_sender_leave = sender.clone();

        motion.connect_enter(move |_, _, _| {
            motion_sender_enter.input(ScreenshotWindowInput::EnterWindow);
        });

        motion.connect_leave(move |_| {
            motion_sender_leave.input(ScreenshotWindowInput::LeaveWindow);
        });

        overlay.add_controller(motion);

        window.present();

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ScreenshotWindowInput::Draw(surface) => self.draw(Some(surface)),
            ScreenshotWindowInput::Redraw => self.draw(None),
            ScreenshotWindowInput::LeaveWindow => self.toolbar.widget().hide(),
            ScreenshotWindowInput::EnterWindow => self.toolbar.widget().show(),
            ScreenshotWindowInput::ToolbarEvent(event) => sender
                .output_sender()
                .emit(ScreenshotWindowOutput::ToolbarEvent(event)),
        }
    }
}
