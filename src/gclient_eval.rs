use std::{collections::HashMap, path::Path};

use serde::Serialize;

/// Safely parses a local configuration file (like .gclient or
/// .gclient_entries) containing only assignments of literals, lists,
/// dicts, and basic operations.
pub(crate) fn parse_local_config(content: &str, filename: impl AsRef<Path>) -> LocalConfig {
    // TODO: the syntax is limited, so it's possible to use winnow to parse.
    todo!()
}

// TODO: either split into different parser, or?
#[derive(Clone, Debug, Serialize)]
pub(crate) struct LocalConfig {
    pub entries: HashMap<String, String>,
}
