use clap::{Parser, Subcommand};

use depot_tools_rs::{fetch::Fetch, gclient::Gclient};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Fetch(fetch) => fetch.run(),
        Commands::Gclient(_gclient) => {}
    }
}

/// This script can be used to download the Chromium sources. See
/// http://www.chromium.org/developers/how-tos/get-the-code
/// for full usage instructions.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Fetch(Fetch),
    Gclient(Gclient),
}
