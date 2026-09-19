use std::{collections::HashMap, path::PathBuf};

use either::Either;
use serde::Serialize;

/// Only "gclient_git" is actually used, so instead of enum we use struct.
/// This is actually `Spec::GclientGit`.
#[derive(Debug, Default, Serialize)]
pub struct Spec {
    pub solutions: Vec<Solution>,
    pub target_os: Vec<String>,
    pub target_os_only: String,
    pub cache_dir: Option<PathBuf>,
    pub cache_mode: Option<String>,
    pub with_branch_heads: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct Solution {
    pub name: String,
    pub url: String,
    pub deps_file: String,
    // TODO: value can be object
    pub custom_deps: HashMap<String, String>,
    pub custom_vars: HashMap<String, String>,
    pub protocol_override: Option<String>,
}

impl Default for Solution {
    fn default() -> Self {
        Self {
            deps_file: "DEPS".to_string(),
            name: Default::default(),
            url: Default::default(),
            custom_deps: Default::default(),
            custom_vars: Default::default(),
            protocol_override: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Alias {
    pub config: String,
    pub props: HashMap<String, String>,
}

pub type SpecOrAlias = Either<Spec, Alias>;

/// Trait for all configs.
pub trait Config {
    fn fetch_spec(&self, props: &HashMap<String, String>) -> SpecOrAlias;
    fn expected_roots(&self, props: &HashMap<String, String>) -> String;
}
