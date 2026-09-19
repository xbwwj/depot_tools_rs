use std::{env::current_dir, fs, path::Path};

pub(crate) fn is_env_cog() -> bool {
    current_dir().unwrap().starts_with("/google/cog/cloud")
}

pub(crate) fn file_read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).unwrap()
}
