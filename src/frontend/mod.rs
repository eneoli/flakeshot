use gtk4::CssProvider;
use relm4::RelmApp;
use tracing::{debug, error};

use crate::{daemon, frontend::main_window::AppModel, get_socket_file_path};

pub mod file_chooser;
pub mod main_window;
pub mod screenshot_window;
pub mod ui;
pub mod ui_manager;

/// Starts the system tray of flakeshot and its daemon
pub fn start() -> anyhow::Result<()> {
    let Some(_lock_guard) = daemon::acquire_lock()? else {
        return Err(daemon::Error::AlreadyRunning.into());
    };

    // there's no daemon yet => remove the socket file to be able to create a new one
    {
        let sock_path = get_socket_file_path();
        if let Err(e) = std::fs::remove_file(sock_path.clone()) {
            if e.kind() != std::io::ErrorKind::NotFound {
                error!(
                    "Couldn't remove socket file '{}': {}",
                    sock_path.to_string_lossy(),
                    e
                );
                return Err(daemon::Error::IO(e).into());
            }
        }
        debug!("Old socket path successfully removed");
    }

    let app = RelmApp::new("org.flakeshot.app")
        .with_args(vec![])
        .visible_on_activate(false);

    relm4_icons::initialize_icons();
    initialize_css();

    app.run::<AppModel>(());

    Ok(())
}

fn initialize_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("../frontend/style.css"));

    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
