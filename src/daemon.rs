use std::fs::File;

use tracing_unwrap::ResultExt;

use crate::XDG;

const LOCK_FILE: &str = "daemon.lock";

pub fn start() {
    let Some(_lock_file) = aquire_lock_file() else {
        std::process::exit(0)
    };
}

fn aquire_lock_file() -> Option<File> {
    let path = XDG
        .get()
        .unwrap()
        .place_runtime_file(LOCK_FILE)
        .expect_or_log("Couldn't get lock file path");

    let lock_file = File::open(path).expect_or_log("Couldn't open lock file");

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
