use std::{fs::File, io::Write, os::unix::net::UnixStream};

use anyhow::Context;
use gtk4::CssProvider;
use relm4::RelmApp;
use tokio::{io::Interest, net::UnixListener};
use tracing::{debug, error, info};

pub mod message;

use crate::{frontend::main_window::AppModel, get_socket_file_path, get_xdg};

use self::message::Message;

const LOCK_FILE: &str = "daemon.lock";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Couldn't aquire the socket: {0}")]
    AquireSocket(rustix::io::Errno),

    #[error("The daemon isn't running yet. Please start it. (See help page.)")]
    NotRunning,
}

#[tracing::instrument]
pub fn start() -> anyhow::Result<()> {
    let _lock_guard = aquire_lock()?;
    debug!("Starting daemon");

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

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(_start())?;

    Ok(())
}

#[tracing::instrument]
async fn _start() -> anyhow::Result<()> {
    let listener = {
        let socket_path = get_socket_file_path();
        UnixListener::bind(socket_path).context("Couldn't bind to socket.")?
    };
    debug!("Socket listener created");

    let (stream, _addr) = listener
        .accept()
        .await
        .context("Can't start accepting listeners on socket")?;

    let mut buffer: Vec<u8> = Vec::new();

    loop {
        let _ = stream.ready(Interest::READABLE).await?;

        match stream.try_read_buf(&mut buffer) {
            Ok(0) => return Ok(()), // socket got closed for whatever reason
            Ok(_) => process_message(&mut buffer),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        };

        break;
    }

    Ok(())
}

#[tracing::instrument]
fn aquire_lock() -> anyhow::Result<Option<File>> {
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
            return Err(Error::AquireSocket(err).into());
        }
    }

    Ok(Some(lock_file))
}

pub fn send_message(msg: Message) -> anyhow::Result<()> {
    if aquire_lock()?.is_some() {
        info!("Daemon is not running.");
        return Err(Error::NotRunning.into());
    }

    let socket_path = get_socket_file_path();
    let mut stream =
        UnixStream::connect(socket_path).context("Couldn't conenct to daemon socket")?;

    let msg_string = ron::to_string(&msg)?;
    stream
        .write_all(msg_string.as_bytes())
        .context("Couldn't write message to daemon socket")?;

    Ok(())
}

fn process_message(buffer: &mut Vec<u8>) {
    let msg: Message = {
        let bytes = std::mem::take(buffer);
        let string = String::from_utf8(bytes).unwrap();
        ron::from_str(&string).unwrap()
    };

    match msg {
        Message::CreateScreenshot => start_gui(),
    }
}

#[tracing::instrument]
fn start_gui() {
    let app = RelmApp::new("org.flakeshot.app").with_args(vec![]);
    relm4_icons::initialize_icons();
    initialize_css();

    app.run::<AppModel>(());
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
