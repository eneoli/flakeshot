use std::{cell::RefCell, rc::Rc};

use crate::backend::OutputInfo;
use gdk4::prelude::MonitorExt;
use gtk4_layer_shell::LayerShell;
use relm4::{
    drawing::DrawHandler,
    gtk::{
        self,
        prelude::{GtkWindowExt, WidgetExt},
    },
    ComponentParts, ComponentSender, SimpleComponent,
};

use super::{main_window::AppModel, ui::canvas::Canvas};

#[derive(Debug)]
pub struct ScreenshotWindowModel {
    output_info: OutputInfo,
    monitor: gdk4::Monitor,
    draw_handler: DrawHandler,
    canvas: Rc<RefCell<Canvas>>,
}

pub struct ScreenshotWindowInit {
    pub output_info: OutputInfo,
    pub monitor: gdk4::Monitor,
    pub parent_sender: Rc<relm4::ComponentSender<AppModel>>,
    pub canvas: Rc<RefCell<Canvas>>,
}

#[derive(Debug)]
pub enum ScreenshotWindowInput {
    Redraw,
}

impl ScreenshotWindowModel {
    fn draw(&mut self) {
        let x = self.monitor.geometry().x() as f64;
        let y = self.monitor.geometry().y() as f64;
        let width = self.monitor.geometry().width();
        let height = self.monitor.geometry().height();

        let ctx = self.draw_handler.get_context();

        let canvas = self.canvas.borrow();
        let surface_portion = canvas
            .crop(x, y, width, height)
            .expect("Couldn't get surface portion.");

        ctx.set_source_surface(&surface_portion, 0.0, 0.0)
            .expect("Couldn't set source surface.");

        ctx.paint().expect("Couldn't paint.");
    }
}

impl SimpleComponent for ScreenshotWindowModel {
    type Input = ScreenshotWindowInput;
    type Output = ();
    type Init = ScreenshotWindowInit;
    type Root = gtk::Window;
    type Widgets = ();

    fn init_root() -> Self::Root {
        let window = gtk::Window::new();
        window.init_layer_shell();
        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        window.set_anchor(gtk4_layer_shell::Edge::Left, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_layer(gtk4_layer_shell::Layer::Overlay);

        window
    }

    fn init(
        payload: ScreenshotWindowInit,
        window: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = ScreenshotWindowModel {
            output_info: payload.output_info,
            monitor: payload.monitor,
            draw_handler: DrawHandler::new(),
            canvas: payload.canvas,
        };

        window.hide(); // unrealize window to prevent wayland protocol error when resizing

        window.set_monitor(&model.monitor);
        window.set_default_size(
            model.monitor.geometry().width(),
            model.monitor.geometry().height(),
        );

        let width = model.output_info.width as i32;
        let height = model.output_info.height as i32;

        let overlay = gtk::Overlay::new();

        let drawing_area = model.draw_handler.drawing_area();
        drawing_area.set_size_request(width, height);
        drawing_area.set_vexpand(true);
        drawing_area.set_hexpand(true);

        overlay.add_overlay(drawing_area);

        window.set_child(Some(&overlay));

        window.connect_realize(move |_| {
            let s = sender.clone();

            // make sure window is finished rendering
            gtk::glib::idle_add(move || {
                s.input(ScreenshotWindowInput::Redraw);
                gtk::glib::ControlFlow::Continue
            });
        });

        window.present();

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {
        self.draw();
    }
}
