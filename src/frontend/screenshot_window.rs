use crate::backend::OutputInfo;
use cairo::glib::Bytes;
use gdk4::prelude::MonitorExt;
use gdk_pixbuf::Colorspace;
use gtk4_layer_shell::LayerShell;
use image::DynamicImage;
use relm4::{
    gtk::{
        self,
        prelude::{GtkWindowExt, WidgetExt},
    },
    ComponentParts, ComponentSender, SimpleComponent,
};

pub struct ScreenshotWindowModel {
    output_info: OutputInfo,
    image: DynamicImage,
    monitor: gdk4::Monitor,
}

pub struct ScreenshotWindowInit {
    pub output_info: OutputInfo,
    pub image: DynamicImage,
    pub monitor: gdk4::Monitor,
}

impl SimpleComponent for ScreenshotWindowModel {
    type Input = ();
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
        _sender: ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = ScreenshotWindowModel {
            output_info: payload.output_info,
            image: payload.image,
            monitor: payload.monitor,
        };

        window.hide(); // unrealize window to prevent wayland protocol error when resizing

        window.set_default_size(
            model.monitor.geometry().width(),
            model.monitor.geometry().height(),
        );
        window.set_monitor(&model.monitor);

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

        window.set_child(Some(&picture));
        window.show();

        ComponentParts { model, widgets: () }
    }
}
