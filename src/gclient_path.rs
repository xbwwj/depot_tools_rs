use std::{
    collections::HashSet,
    env,
    fs::{self, canonicalize},
    path::{Path, PathBuf},
};

use cached::cached;

use crate::{
    gclient_eval::{self, LocalConfig},
    gclient_utils,
};

/// Tries to find the gclient root.
#[cached]
pub(crate) fn find_gclient_root(from_dir: &Path, filename: Option<&str>) -> Option<PathBuf> {
    let filename = filename.unwrap_or(".gclient");

    let real_from_dir = canonicalize(from_dir).unwrap();

    // recursively parent to find .gclient
    let mut path = real_from_dir.clone();
    while !path.join(filename).exists() {
        if !path.pop() {
            return None;
        }
    }

    // TODO: log

    // if .gclient is just in from_dir
    if path == real_from_dir {
        return Some(path);
    }

    // check if there is .gclient_entries
    let entries_filename = path.join(format!("{}_entries", filename));
    if !entries_filename.exists() {
        if !gclient_utils::is_env_cog() {
            eprintln!(
                "{} missing, {} file in parent directory {} might not be the file you want to use.",
                entries_filename.display(),
                filename,
                path.display()
            );
        }
        return Some(path);
    }

    let entries_content = gclient_utils::file_read(&entries_filename);
    let scope = gclient_eval::parse_local_config(&entries_content, &entries_filename);

    let real_path = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());

    let all_directories: HashSet<PathBuf> = scope
        .entries
        .keys()
        .filter_map(|k| {
            let mut k_path = path.clone();
            for part in k.split('/') {
                k_path.push(part);
            }
            let real_k_path = fs::canonicalize(&k_path).unwrap_or(k_path);
            real_k_path
                .strip_prefix(&real_path)
                .ok()
                .map(|p| p.to_path_buf())
        })
        .collect();

    let real_from = fs::canonicalize(&real_from_dir).unwrap_or_else(|_| real_from_dir.clone());
    let relative_from = real_from.strip_prefix(&real_path).ok()?;

    let mut path_to_check: &Path = relative_from;
    while !path_to_check.as_os_str().is_empty() {
        if all_directories.contains(path_to_check) {
            return Some(path);
        }
        match path_to_check.parent() {
            Some(parent) => path_to_check = parent,
            None => break,
        }
    }

    None
}

/// Returns the parsed .gclient config contents.
pub(crate) fn get_gclient_config(
    gclient_root_dir_path: Option<&Path>,
    filename: Option<&str>,
) -> Option<LocalConfig> {
    let filename = filename.unwrap_or(".gclient");

    let root_dir = gclient_root_dir_path.map(|p| p.to_path_buf()).or_else(|| {
        let cwd = env::current_dir().ok()?;
        find_gclient_root(&cwd, Some(filename))
    })?;

    _get_gclient_config_inner(&root_dir, filename)
}

#[cached]
fn _get_gclient_config_inner(root_dir: &Path, filename: &str) -> Option<LocalConfig> {
    let config_file = root_dir.join(filename);
    if !config_file.exists() {
        return None;
    }
    let contents = gclient_utils::file_read(&config_file);
    Some(gclient_eval::parse_local_config(&contents, config_file))
}
