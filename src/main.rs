use flakeshot::backend;
use gio::prelude::*;
use glib::{RustClosure, Bytes};
use gtk::{prelude::*, BoxLayout, Orientation, Label, gdk::Key};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

// https://github.com/wmww/gtk-layer-shell/blob/master/examples/simple-example.c
fn activate(application: &gtk::Application) {
    // Create a normal GTK window however you like
    let window = gtk::ApplicationWindow::new(application);

    // Before the window is first realized, set it up to be a layer surface
    window.init_layer_shell();
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
    //window.set_fullscreened(true);
    window.set_default_size(1920, 1080);

    // Display above normal windows
    window.set_layer(Layer::Overlay);

    let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed((|value, key, _, _| {
        if key == Key::Escape {
            std::process::exit(0);
        }
        glib::Propagation::Proceed
    }));

    window.add_controller(key_controller);

    // Push other windows out of the way
    //window.auto_exclusive_zone_enable();

    // The margins are the gaps around the window's edges
    // Margins and anchors can be set like this...
    //window.set_margin(Edge::Left, 0);
    //window.set_margin(Edge::Right, 0);
    //window.set_margin(Edge::Top, 0);
    //window.set_margin(Edge::Bottom, 0);

    // ... or like this
    // Anchors are if the window is pinned to each edge of the output
    let anchors = [
        (Edge::Left, true),
        (Edge::Right, false),
        (Edge::Top, true),
        (Edge::Bottom, false),
    ];

    for (anchor, state) in anchors {
        //window.set_anchor(anchor, state);
    }


    let screenshots = backend::wayland::create_screenshots().unwrap();

    let screen = screenshots[0].1.clone();

    let pixels = screen.into_bytes();
    let bytes = Bytes::from(&pixels);

    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
        &bytes,
        gtk::gdk_pixbuf::Colorspace::Rgb,
        true, 8, 1920, 1080, 1920 * 4 
    );
    
    let pixbuf2 = gtk::gdk_pixbuf::Pixbuf::from_file("/home/oliver/pikatchu.png").unwrap();

    let layout = gtk::Overlay::new();//gtk::Box::new(Orientation::Vertical, 0);
    
    let img = gtk::Picture::new(); //from_pixbuf(Some(&pixbuf2));

    img.set_pixbuf(Some(&pixbuf));

    layout.add_overlay(&img);

    let label = Label::new(None);
    label.set_markup("<span font_desc=\"50.0\">flakeshot - Work In Progress</span>");
    layout.add_overlay(&label);

    //layout.append(&label);
    window.set_child(Some(&layout));
    //window.set_child(Some(&label));
    window.fullscreen();
    window.show();
    window.fullscreen()
}

fn main() {
    let application = gtk::Application::new(Some("org.flakeshot"), Default::default());

    application.connect_activate(|app| {
        activate(app);
    });

    application.run();
}

