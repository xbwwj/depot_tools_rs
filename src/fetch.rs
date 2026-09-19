use std::{
    collections::HashMap,
    env::{self, current_dir},
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Args;
use either::Either;

use crate::{
    caffeinate,
    fetch_configs::chromium::Chromium,
    fetch_util::{Config, Spec},
    gclient_utils, git_common, utils,
};

/// fetch
#[derive(Args, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Fetch {
    /// Don't run commands, only print them.
    #[arg(short = 'n', long = "dry-run", default_value_t = false)]
    pub dry_run: bool,

    /// Don't run hooks after checkout.
    #[arg(long = "no-hooks", alias = "nohooks", default_value_t = false)]
    pub no_hooks: bool,

    /// Perform shallow clones, don't fetch the full git history.
    #[arg(long = "no-history", alias = "nohistory", default_value_t = false)]
    pub no_history: bool,

    /// Speed up the initial checkout by bootstrapping each clone from
    /// a local git cache seeded with a prebuilt snapshot. The resulting
    /// checkout is configured like a normal one and does not depend on the
    /// cache afterwards. The cache directory is chosen automatically
    /// unless $GIT_CACHE_PATH is set.
    #[arg(long = "git-cache", default_value_t = false)]
    pub git_cache: bool,

    /// (dangerous) Don't look for existing .gclient file.
    #[arg(long = "force", default_value_t = false)]
    pub force: bool,

    /// Protocol to use to fetch dependencies, defaults to https.
    #[arg(short = 'p', long = "protocol-override")]
    pub protocol_override: Option<String>,

    /// On macOS, prevent idle sleep during the operation. Enabled by default.
    /// Use --caffeinate=false to disable. No effect on other platforms.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub caffeinate: bool,

    /// Project to fetch, e.g. chromium.
    pub config: String,

    /// Additional properties or arguments.
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        num_args = 0..,
        value_parser = parse_key_value
    )]
    pub props: HashMap<String, String>,
}

impl Fetch {
    pub(crate) fn run(self) {
        let _guard = caffeinate::scope(true);
        let (spec, root) = run_config_fetch(&self.config, &self.props, None).unwrap();
        run(self, spec, root);
    }
}

fn parse_key_value(kv: &str) -> Result<(String, String), String> {
    if let Some((key, value)) = kv.trim_start_matches('-').split_once('=') {
        if key.is_empty() {
            return Err(format!("Got bad argument {kv}"));
        }
        Ok((key.to_string(), value.to_string()))
    } else {
        Err(format!("invalid format: {}", kv))
    }
}

/// Invoke a config's fetch method with the passed-through args
/// and return its json output.
fn run_config_fetch(
    config: &str,
    props: &HashMap<String, String>,
    aliased: Option<bool>,
) -> Result<(Spec, String), String> {
    let aliased = aliased.unwrap_or(false);
    let config: Box<dyn Config> = match config {
        "chromium" => Box::new(Chromium),
        _ => return Err(format!("Could not find a config for {config}")),
    };
    let spec = match config.fetch_spec(props) {
        Either::Left(spec) => spec,
        Either::Right(alias) => {
            // alias should only be used once
            assert!(!aliased);

            let mut merged_props = HashMap::new();
            merged_props.extend(alias.props.into_iter());
            merged_props.extend(props.clone().into_iter());

            return run_config_fetch(&alias.config, &merged_props, Some(true));
        }
    };
    let root = config.expected_roots(props);
    Ok((spec, root))
}

/// Perform a checkout with the give type and configuration.
///
/// # Arguments
///
/// - `options`: Options instance.
/// - `spec`: Checkout configuration returned by the config's fetch_spec method (checkout type, respository url, etc.).
/// - `root`: The directory into which the repo expects to be checkout out.
fn run(options: Fetch, mut spec: Spec, root: String) -> ExitCode {
    if gclient_utils::is_env_cog() {
        eprintln!(
            r#"Your current directory appears to be in a Cog workspace.
"fetch" command is not supported in this environment."#
        );
        return ExitCode::FAILURE;
    }

    if options.git_cache {
        spec.cache_dir.get_or_insert_with(|| {
            env::var("GIT_CACHE_PATH")
                .map(PathBuf::from)
                .unwrap_or(utils::depot_tools_cache_dir().join("git_cache"))
        });
        spec.cache_mode.get_or_insert("bootstrap".into());
    }

    if let Some(protocol_override) = &options.protocol_override {
        for solution in spec.solutions.iter_mut() {
            solution.protocol_override = Some(protocol_override.clone());
        }
    }

    let force = options.force;
    let checkout = Checkout::new(options, spec, root);

    if !force && checkout.exists() {
        println!(
            r#"Your current directory appears to already contain, or be part of
a checkout. "fetch" is used only to get new checkouts. Use 
"gclient sync" to update existing checkouts.

Fetch also does not yet deal with partial checkouts, so if fetch
failed, delete the checkout and start over (crbug.com/230681)."#
        );
        return ExitCode::FAILURE;
    }

    return checkout.init();
}

struct Checkout {
    base: PathBuf,
    options: Fetch,
    spec: Spec,
    root: String,
}

impl Checkout {
    /// checkout type is always gclient-git
    fn new(options: Fetch, spec: Spec, root: String) -> Self {
        let base = current_dir().unwrap();
        Self {
            base,
            options,
            spec,
            root,
        }
    }

    fn exists(&self) -> bool {
        match self.run_gclient(&["root"]) {
            Ok(gclient_root) => {
                fs::exists(PathBuf::from(gclient_root).join(".gclient")).unwrap()
                    || fs::exists(
                        PathBuf::from(current_dir().unwrap())
                            .join(&self.root)
                            .join(".git"),
                    )
                    .unwrap()
            }
            Err(_) => fs::exists(PathBuf::from(current_dir().unwrap()).join(&self.root)).unwrap(),
        }
    }

    fn init(&self) -> ExitCode {
        // gclient config
        _ = self.run_gclient(&[
            "config",
            "--spec",
            &serde_json::to_string(&self.spec).unwrap(),
        ]);

        // gclient sync
        let mut sync_cmd = vec!["sync"];
        if self.options.no_hooks {
            sync_cmd.push("--nohooks");
        }
        if self.options.no_history {
            sync_cmd.push("--nohistory");
        }
        if self.spec.with_branch_heads.unwrap_or_default() {
            sync_cmd.push("--with_branch_heads");
        }
        _ = self.run_gclient(&sync_cmd);

        // configure git
        let wd = self.base.join(&self.root);
        if self.options.dry_run {
            println!("cd {}", wd.display());
        }
        if !self.options.no_history {
            self.run_git(
                &[
                    "config",
                    "--add",
                    "remote.origin.fetch",
                    "+refs/tags/*:refs/tags/*",
                ],
                &wd,
            );
        }
        self.run_git(&["config", "diff.ignoreSubmodules", "dirty"], &wd);

        ExitCode::SUCCESS
    }

    fn run_gclient(&self, _cmd: &[&str]) -> Result<String, io::Error> {
        // TODO: implement gclient
        Ok("".to_string())
    }

    fn run_git(&self, cmd: &[&str], cwd: &Path) -> String {
        println!("Running git {}", cmd.join(" "));
        if self.options.dry_run {
            return String::new();
        }
        git_common::run(cmd, Some(cwd))
    }
}
