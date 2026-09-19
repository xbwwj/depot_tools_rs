use std::thread::available_parallelism;

use clap::{Args, Subcommand};

use crate::gclient_path;

/// gclient
#[derive(Args, Debug)]
pub struct Gclient {
    /// Specify how many SCM commands can run in parallel.
    #[arg(
        long = "jobs",
        short = 'j',
        default_value_t = available_parallelism().unwrap().get()
    )]
    jobs: usize,

    /// Produces additional output for diagnostics.
    /// Can be used up to three times for more logging info.
    #[arg(long = "verbose", short = 'v')]
    verbose: bool,

    #[command(subcommand)]
    command: Commnads,
}

#[derive(Debug, Subcommand)]
enum Commnads {
    Grep(CmdGrep),
    Root(CmdRoot),
    Config(CmdConfig),
    Pack(CmdPack),
    Status(CmdStatus),
    Sync(CmdSync),
    Validate(CmdValidate),
    Diff(CmdDiff),
    Revert(CmdRevert),
    Runhooks(CmdRunhooks),
    Installhooks(CmdInstallhooks),
    Revinfo(CmdRevinfo),
    Getconfig(CmdGetconfig),
    Getdep(CmdGetdep),
    Setdep(CmdSetdep),
    Verify(CmdVerify),
    Metrics(CmdMetrics),
}

/// Grep through git repos managed by gclient.
#[derive(Args, Debug)]
struct CmdGrep {}

/// Outputs the solution root (or current dir if there isn't one).
#[derive(Args, Debug)]
struct CmdRoot {}

/// Create a .gclient file in the current directory.
#[derive(Args, Debug)]
struct CmdConfig {}

/// Generate a patch which can be applied at the root of the tree.
#[derive(Args, Debug)]
struct CmdPack {}

/// Show modification status for every dependencies.
#[derive(Args, Debug)]
struct CmdStatus {}

/// Checkout/update all modules.
#[derive(Args, Debug)]
struct CmdSync {}

/// Validate the .gclient and DEPS syntax.
#[derive(Args, Debug)]
struct CmdValidate {}

/// Displays local diff for every dependencies.
#[derive(Args, Debug)]
struct CmdDiff {}

/// Revert all modifications in every dependencies.
#[derive(Args, Debug)]
struct CmdRevert {}

/// Runs hooks for files that have been modified in the local working copy.
#[derive(Args, Debug)]
struct CmdRunhooks {}

/// Installs gclient git gooks.
#[derive(Args, Debug)]
struct CmdInstallhooks {}

/// Outputs revision info mapping for the client and its dependencies.
#[derive(Args, Debug)]
struct CmdRevinfo {}

/// Get config values from the .gclient file.
#[derive(Args, Debug)]
struct CmdGetconfig {}

impl CmdGetconfig {
    fn run(&self) {
        // TODO: use option configname
        let config = gclient_path::get_gclient_config(None, None);
        if config.is_none() {
            panic!("Could not find configuration file.");
        }

        println!("{}", serde_json::to_string_pretty(&config).unwrap());
    }
}

/// Get revision information and variable values from a DEPS file.
#[derive(Args, Debug)]
struct CmdGetdep {}

/// Modifies dependency revisions and variable values in a DEPS file.
#[derive(Args, Debug)]
struct CmdSetdep {}

/// Verifies the DEPS file deps are only from allowed_hosts.
#[derive(Args, Debug)]
struct CmdVerify {}

/// Report, and optionally modifies, the status of metric collection.
#[derive(Args, Debug)]
struct CmdMetrics {}
