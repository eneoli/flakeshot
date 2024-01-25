use anyhow::Context;
use gtk4::CssProvider;
use relm4::RelmApp;
use tokio::{io::Interest, net::UnixListener};

pub mod message;

use crate::{frontend::main_window::AppModel, get_socket_file_path, SOCKET_FILENAME, XDG};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Couldn't aquire the socket: {0}")]
    AquireSocket(rustix::io::Errno),
}

pub fn start() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(_start())?;

    Ok(())
}

async fn _start() -> anyhow::Result<()> {
    let listener = aquire_socket().await?;
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
        }
    }
}

async fn aquire_socket() -> anyhow::Result<UnixListener> {
    let socket_path = get_socket_file_path();
    let socket = UnixListener::bind(socket_path.clone()).context(
        format!(
            "Couldn't bind to the given socket: {}",
            socket_path.to_string_lossy()
        )
        .leak(),
    )?;

    if let Err(err) = rustix::fs::flock(
        &socket,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    ) {
        let daemon_already_exists = err == rustix::io::Errno::WOULDBLOCK;

        if daemon_already_exists {
            std::process::exit(0);
        } else {
            return Err(Error::AquireSocket(err).into());
        }
    };

    Ok(socket)
}

fn process_message(buffer: &mut Vec<u8>) {
    tracing::debug!("success!");
}

fn start_gui() {
    let app = RelmApp::new("org.flakeshot.app");
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
