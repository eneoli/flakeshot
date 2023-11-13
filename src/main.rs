use clap::Parser;
use flakeshot::cli::Cli;

async fn run() -> Result<(), ()> {
    let _cli = Cli::parse();

    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap();
}
