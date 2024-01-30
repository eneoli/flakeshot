use anyhow::Context;
use relm4::Sender;
use tokio::{io::Interest, net::UnixListener};
use tracing::{debug, error};

use crate::get_socket_file_path;

use super::Message;

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
