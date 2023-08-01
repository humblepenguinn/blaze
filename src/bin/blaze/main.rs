use clap::Parser;

pub mod actions;
pub mod cli;
pub mod error;
pub mod version;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    cli.command.run().await;
}
