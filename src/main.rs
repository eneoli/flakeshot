use clap::Parser;
use flakeshot::{cli::Cli, tray};

async fn run() -> Result<(), ()> {
    let _cli = Cli::parse();

    tray::start();

    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap();
}
