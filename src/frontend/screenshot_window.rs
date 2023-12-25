use crate::backend::OutputInfo;
use arboard::ImageData;
use cairo::glib::Bytes;
use gdk4::prelude::MonitorExt;
use gdk_pixbuf::Colorspace;
use gtk4_layer_shell::LayerShell;
use image::DynamicImage;
use relm4::{
    drawing::DrawHandler,
    gtk::{
        self,
        prelude::{DrawingAreaExtManual, EventControllerExt, GtkWindowExt, WidgetExt},
        GestureClick,
    },
    ComponentParts, ComponentSender, SimpleComponent,
};

#[derive(Debug)]
pub struct ScreenshotWindowModel {
    output_info: OutputInfo,
    image: DynamicImage,
    monitor: gdk4::Monitor,

    draw_handler: DrawHandler,

    isSelecting: bool,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

pub struct ScreenshotWindowInit {
    pub output_info: OutputInfo,
    pub image: DynamicImage,
    pub monitor: gdk4::Monitor,
}

#[derive(Debug)]
pub enum InputMsg {
    MouseMove { x: f64, y: f64 },
    MousePress { x: f64, y: f64 },
    MouseRelease { x: f64, y: f64 },
}

impl SimpleComponent for ScreenshotWindowModel {
    type Input = InputMsg;
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
            image: payload.image,
            monitor: payload.monitor,
            isSelecting: false,
            draw_handler: DrawHandler::new(),
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
        };

        window.hide(); // unrealize window to prevent wayland protocol error when resizing

        window.set_default_size(
            model.monitor.geometry().width(),
            model.monitor.geometry().height(),
        );
        window.set_monitor(&model.monitor);

        let cursor = gdk4::Cursor::from_name("cross", None);
        window.set_cursor(cursor.as_ref());

        let gesture = GestureClick::new();
        let connector = model.monitor.connector().clone();
        let connector2 = connector.clone();
        let input_sender_p = sender.input_sender().clone();
        gesture.connect_pressed(move |gesture, _, x, y| {
            input_sender_p.send(InputMsg::MousePress { x, y });
        });

        let input_sender_r = sender.input_sender().clone();
        gesture.connect_released(move |_, _, x, y| {
            input_sender_r.send(InputMsg::MouseRelease { x, y });
        });

        let motion = gtk::EventControllerMotion::new();
        let input_sender = sender.input_sender().clone();
        motion.connect_motion(move |motion, x, y| {
            input_sender.send(InputMsg::MouseMove { x, y }).unwrap();
        });

        window.add_controller(gesture);
        window.add_controller(motion);

        let width = model.output_info.width as i32;
        let height = model.output_info.height as i32;

        let picture = {
            let image_bytes = model.image.as_bytes();
            let data = Bytes::from(image_bytes);

            let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
                &data,
                Colorspace::Rgb,
                true,
                8,
                model.output_info.width as i32,
                model.output_info.height as i32,
                model.output_info.width as i32 * 4,
            );

            gtk::Picture::for_pixbuf(&pixbuf)
        };

        let overlay = gtk::Overlay::new();

        model
            .draw_handler
            .drawing_area()
            .set_size_request(width, height);
        // model.draw_handler.drawing_area().set_draw_func(|_, ctx, width, height| {
        //     ctx.set_source_rgba(0.0, 0.0, 0.0, 0.5);
        //     ctx.rectangle(0f64, 0f64, width as f64, height as f64);
        //     ctx.fill().unwrap();
        // });

        let label = gtk::Label::new(Some("test"));
        overlay.add_overlay(&picture);
        overlay.add_overlay(model.draw_handler.drawing_area());

        window.set_child(Some(&overlay));
        window.show();

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        let width = self.output_info.width as f64;
        let height = self.output_info.height as f64;
        if let InputMsg::MouseMove { x, y } = message {
            if !self.isSelecting {
                return;
            }
            self.x2 = x;
            self.y2 = y;
        }

        if let InputMsg::MousePress { x, y } = message {
            self.x1 = x;
            self.y1 = y;
            self.x2 = x;
            self.y2 = y;
            self.isSelecting = true;
        }

        if let InputMsg::MouseRelease { x, y } = message {
            let cropped = self.image.crop(
                self.x1 as u32,
                self.y1 as u32,
                (self.x2 - self.x1) as u32,
                 (self.y2 - self.y1) as u32,
            );
            let mut clipboard = arboard::Clipboard::new().unwrap();
            let img = ImageData {
                width:  (self.x2 - self.x1) as usize,
                height: (self.y2 - self.y1) as usize,
                bytes:std::borrow::Cow::from(cropped.into_bytes()),
            };

            clipboard.set_image(img).unwrap();

            self.isSelecting = false;
            self.x1 = 0.0;
            self.x2 = 0.0;
            self.y1 = 0.0;
            self.y2 = 0.0;
        }

        let ctx = self.draw_handler.get_context();

        // clear surface
        ctx.set_operator(cairo::Operator::Clear);
        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        ctx.paint_with_alpha(1.0).unwrap();

        // draw gray background
        ctx.set_operator(cairo::Operator::Over);
        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.5);
        ctx.rectangle(0.0, 0.0, width, height);
        ctx.fill().unwrap();

        // ctx.set_operator(cairo::Operator::Clear);
        ctx.set_source_rgb(0.45, 0.6, 0.85);
        ctx.set_line_width(1.0);
        ctx.rectangle(self.x1, self.y1, self.x2 - self.x1, self.y2 - self.y1);
        ctx.stroke().unwrap();
    }
}
