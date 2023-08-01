use clap::{Parser, Subcommand};

#[derive(Parser)]
/// A Minimal And Fast NodeJS Package Manager Written In Rust
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[clap(name = "install", about = "install a new NodeJS package")]
    Install {
        #[clap(value_delimiter = ' ')]
        package_names: Vec<String>,
    },

    #[clap(name = "init", about = "initialize a new NodeJS project")]
    Init {},

    #[clap(name = "version", about = "Print the version")]
    Version {
        #[clap(required = false)]
        verbose: Option<bool>,
    },
}
