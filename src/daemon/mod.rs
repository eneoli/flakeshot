use anyhow::Context;
use tokio::{
    io::Interest,
    net::{UnixListener, UnixStream},
};

pub mod message;

use crate::{SOCKET_PATH, XDG};

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
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }
    }
}

async fn aquire_socket() -> anyhow::Result<UnixListener> {
    let path = XDG
        .get()
        .unwrap()
        .place_runtime_file(SOCKET_PATH)
        .expect("Couldn't get lock file path");

    let socket = UnixListener::bind(path.clone()).context(
        format!(
            "Couldn't bind to the given socket: {}",
            path.to_string_lossy()
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
