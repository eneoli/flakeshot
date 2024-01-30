mod daemon;
mod error;
mod frontend;
mod message;
mod tray;

use anyhow::Context;
use gtk4::CssProvider;
use relm4::RelmApp;
use std::fs::File;
use tracing::{debug, error, info};

pub use error::Error;
pub use message::Message;

use crate::{get_socket_file_path, get_xdg, tray_daemon::frontend::main_window::TrayDaemon};

const LOCK_FILE: &str = "daemon.lock";

pub fn start() -> anyhow::Result<()> {
    let Some(_lock_guard) = acquire_lock()? else {
        return Err(Error::AlreadyRunning.into());
    };
    debug!("Starting tray");

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
                return Err(Error::IO(e).into());
            }
        }
        debug!("Old socket path successfully removed");
    }

    let app = RelmApp::new("org.flakeshot.app")
        .with_args(vec![])
        .visible_on_activate(false);

    relm4_icons::initialize_icons();
    initialize_css();

    app.run::<TrayDaemon>(());

    Ok(())
}
/// If no error occured: Returns the lock-file (if available), otherwise `None` if the lock file
/// couldn't be aquired.
/// Otherwise the error will be returned.
#[tracing::instrument]
pub fn acquire_lock() -> anyhow::Result<Option<File>> {
    let lock_file_path = get_xdg().place_runtime_file(LOCK_FILE).unwrap();

    let lock_file = File::create(lock_file_path).context("Create daemon lock file")?;
    if let Err(err) = rustix::fs::flock(
        &lock_file,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    ) {
        let daemon_already_exists = err == rustix::io::Errno::WOULDBLOCK;

        if daemon_already_exists {
            info!("Daemon is already running");
            return Ok(None);
        } else {
            error!("Couldn't acquire lock: {}", err);
            return Err(Error::AcquireSocket(err).into());
        }
    }

    Ok(Some(lock_file))
}

fn initialize_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("./frontend/style.css"));

    gtk4::style_context_add_provider_for_display(
        &gdk4::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
