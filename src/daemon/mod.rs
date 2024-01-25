use std::fs::File;

use tokio::{io::Interest, net::UnixStream, runtime::Runtime};

pub mod message;

use crate::{SOCKET_PATH, XDG};

const LOCK_FILE: &str = "daemon.lock";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub fn start() -> Result<(), Error> {
    let Some(_lock_file) = aquire_lock_file() else {
        std::process::exit(0)
    };

    let rt = Runtime::new()?;
    rt.block_on(_start())?;

    Ok(())
}

async fn _start() -> Result<(), Error> {
    let socket = UnixStream::connect(SOCKET_PATH.get().unwrap()).await?;

    loop {
        let ready = socket
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;

        if ready.is_readable() {}

        if ready.is_writable() {}
    }

    Ok(())
}

fn aquire_lock_file() -> Option<File> {
    let path = XDG
        .get()
        .unwrap()
        .place_runtime_file(LOCK_FILE)
        .expect("Couldn't get lock file path");

    let lock_file = File::open(path.clone()).unwrap_or_else(|e| {
        panic!(
            "Couldn't open lock file '{}'. Error: {}",
            path.to_string_lossy(),
            e
        )
    });

    if let Err(err) = rustix::fs::flock(
        &lock_file,
        rustix::fs::FlockOperation::NonBlockingLockExclusive,
    ) {
        if err == rustix::io::Errno::WOULDBLOCK {
            return None;
        } else {
            let msg = format!(
                "An error occured while trying to acquire the lock. Error code: {}",
                err
            );

            tracing::error!(msg);
            panic!("{}", msg);
        }
    };

    Some(lock_file)
}
