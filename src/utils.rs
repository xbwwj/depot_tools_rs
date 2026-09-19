use std::path::PathBuf;

use dirs::cache_dir;

/// Returns the per-user cache directory for depot_tools.
///
/// Unlike depot_tools_config_dir(), this never points inside the depot_tools
/// install directory: caches (e.g. git mirrors, virtualenvs) can grow to many
/// GB and are regenerable, so they belong in the platform's user cache area
/// alongside vpython's own `~/.cache/vpython-root.<uid>`.
pub fn depot_tools_cache_dir() -> PathBuf {
    // TODO: should we use depot_tools_rs instead?
    cache_dir().unwrap().join("depot_tools")
}
