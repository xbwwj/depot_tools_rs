use clap::{Parser, Subcommand};

use crate::fetch::Fetch;

mod caffeinate;
mod fetch;
mod fetch_configs;
mod fetch_util;
mod gclient_utils;
mod git_common;
mod utils;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Fetch(fetch) => fetch.run(),
        Commands::Gclient => {}
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
    Gclient,
}
