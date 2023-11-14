use std::fs::File;

use clap::Parser;
use flakeshot::{backend, cli::Cli};
use image::ImageFormat;

async fn run() -> Result<(), ()> {
    let _cli = Cli::parse();

    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap();
}
