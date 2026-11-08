use std::{
    env::current_dir,
    fs,
    path::Path,
    sync::{LazyLock, Mutex},
};

use chrono::{DateTime, Local};
use ctor::ctor;
use regex::Regex;
use rustyline::{DefaultEditor, error::ReadlineError};

pub const RETRY_MAX: usize = 1;
pub const RETRY_INITIAL_SLEEP: u64 = 2;

#[ctor(unsafe)]
pub static START: DateTime<Local> = Local::now();

static _WARNINGS: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub const THREADED_INDEX_PACK_BLOCKLIST: [&str; 1] =
    ["https://chromium.googlesource.com/chromium/reference_builds/chrome_win.git"];

pub fn elapsed(until: Option<DateTime<Local>>) -> String {
    let until = until.unwrap_or_else(Local::now);

    let duration = until.signed_duration_since(START.clone());
    let micros = duration.num_microseconds().expect("duration overflow");

    const DAY: i64 = 86_400_000_000;

    let days = micros.div_euclid(DAY);
    let remainder = micros.rem_euclid(DAY);

    let hours = remainder / 3_600_000_000;
    let minutes = (remainder / 60_000_000) % 60;
    let seconds = (remainder / 1_000_000) % 60;

    if days == 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        let day_label = if days.abs() == 1 { "day" } else { "days" };
        format!("{days} {day_label}, {hours}:{minutes:02}:{seconds:02}")
    }
}

pub fn print_warnings() {
    let warnings = _WARNINGS.lock().unwrap();

    if !warnings.is_empty() {
        eprintln!("\n\nWarnings:");
        for warning in warnings.iter() {
            eprintln!("{warning}");
        }
    }
}

pub fn add_warning(msg: impl Into<String>) {
    _WARNINGS.lock().unwrap().push(msg.into());
}

pub fn fuzzy_match_repo<'a, I, S>(repo: &str, mut candidates: I) -> Option<String>
where
    I: Iterator<Item = &'a S> + Clone,
    S: AsRef<str> + 'a,
{
    if candidates
        .clone()
        .any(|candidate| candidate.as_ref() == repo)
    {
        return Some(repo.to_string());
    }

    if let Some(stripped) = repo.strip_suffix(".git") {
        if candidates
            .clone()
            .any(|candidate| candidate.as_ref() == stripped)
        {
            return Some(stripped.to_string());
        }
    }

    let with_git = format!("{repo}.git");
    if candidates.any(|candidate| candidate.as_ref() == with_git) {
        return Some(with_git);
    }

    None
}

pub fn split_url_revision(url: &str) -> Result<(String, Option<String>), String> {
    static SSH_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(ssh://(?:[-.\w]+@)?[-\w:\.]+/[-~\w\./]+)(?:@(.+))?").unwrap()
    });
    static USER_AT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\w+@").unwrap());

    let (repo, rev) = if url.starts_with("ssh:") {
        let captures = SSH_URL_RE
            .captures(url)
            .ok_or_else(|| format!("Invalid SSH repository URL '{url}'"))?;

        (
            captures.get(1).unwrap().as_str(),
            captures.get(2).map(|m| m.as_str()),
        )
    } else {
        match url.rsplit_once('@') {
            Some((repo, _rev)) if USER_AT_RE.is_match(url) && !repo.contains('@') => (url, None),
            Some((repo, rev)) => (repo, Some(rev)),
            None => (url, None),
        }
    };

    if repo.starts_with('-') {
        return Err(format!(
            "Invalid repository url '{repo}': url cannot start with '-'"
        ));
    }

    Ok((repo.to_string(), rev.map(str::to_string)))
}

pub fn extract_ref_name<'a>(
    remote: &str,
    full_refs_str: &'a str,
) -> Result<Option<&'a str>, regex::Error> {
    let pattern = format!(r"^refs(/.+)?/(({remote})|(heads)|(tags))/(?P<ref_name>.+)");
    let regex = Regex::new(&pattern)?;
    Ok(regex
        .captures(full_refs_str)
        .and_then(|captures| captures.name("ref_name"))
        .map(|m| m.as_str()))
}

pub fn is_git_sha(revision: &str) -> bool {
    static GIT_SHA_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-fA-F0-9]{6,64}$").unwrap());
    GIT_SHA_RE.is_match(revision)
}

pub fn is_full_git_sha(revision: &str) -> bool {
    static FULL_GIT_SHA_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(?:[a-fA-F0-9]{40}|[a-fA-F0-9]{64})$").unwrap());
    FULL_GIT_SHA_RE.is_match(revision)
}

pub fn is_date_revision(revision: &str) -> bool {
    static DATE_REVISION_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\{.+\}$").unwrap());
    !revision.is_empty() && DATE_REVISION_RE.is_match(revision)
}

pub fn make_date_revision(date: &str) -> String {
    format!("{{{date}}}")
}

pub fn ask_for_data(message: &str) -> Result<String, ReadlineError> {
    let mut editor = DefaultEditor::new()?;
    match editor.readline(message) {
        Ok(line) => Ok(line),
        Err(ReadlineError::Interrupted) => std::process::exit(1),
        Err(err) => Err(err),
    }
}

// TODO: file_read
// TODO: file_write

pub fn is_env_cog() -> bool {
    current_dir().unwrap().starts_with("/google/cog/cloud")
}

pub fn file_read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).unwrap()
}
