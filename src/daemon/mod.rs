use std::fs::File;

use anyhow::Context;
use relm4::Sender;
use tokio::{io::Interest, net::UnixListener};
use tracing::{debug, error, info};

const LOCK_FILE: &str = "daemon.lock";

mod error;
mod message;

pub use error::Error;
pub use message::Message;

use crate::{get_socket_file_path, get_xdg};

pub async fn start(out: Sender<Message>) {
    let listener = {
        let socket_path = get_socket_file_path();
        UnixListener::bind(socket_path)
            .context("Couldn't bind to socket.")
            .unwrap()
    };
    debug!("Socket listener created");

    let mut byte_buffer: Vec<u8> = vec![];
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                if let Err(e) = stream.ready(Interest::READABLE).await {
                    error!(
                        "An IO error occured while waiting for messages of the listener: {}",
                        e
                    );
                }

                match stream.try_read_buf(&mut byte_buffer) {
                    Ok(amount_bytes) if amount_bytes > 0 => process_message(&mut byte_buffer, &out),
                    Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => {
                        error!(
                            "An error occured while trying to read the message from the socket: {}",
                            e
                        );
                    }
                    _ => {}
                };
            }
            Err(e) => error!("Coulnd't connect to listener: {}", e),
        }
    }
}

fn process_message(buffer: &mut Vec<u8>, out: &Sender<Message>) {
    let msg: Message = {
        let bytes = std::mem::take(buffer);
        let string = String::from_utf8(bytes).unwrap();
        ron::from_str(&string).unwrap()
    };

    out.send(msg).unwrap();
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
