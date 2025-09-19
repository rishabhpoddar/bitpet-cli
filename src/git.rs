use crate::utils;
use std::process::Command;
use std::sync::OnceLock;

use crate::error;
use crate::error::WithBacktrace;
use bitpet_cli::track_errors;
// NOTE: These are blocking function calls and are being called in an async context. But it is
// OK cause this is client code anyway.

#[track_errors]
pub fn is_git(normalised_path: &utils::NormalisedGitPath) -> bool {
    let mut path = normalised_path.path();
    if path.join(".git").exists() {
        return true;
    }
    while let Some(parent) = path.parent() {
        if parent.join(".git").exists() {
            return true;
        }
        path = parent;
    }
    false
}

// Thread-safe, lazy-initialized static cache for git username
static CACHED_GIT_USERNAME: OnceLock<String> = OnceLock::new();

#[track_errors]
fn get_git_username() -> Result<String, GitError> {
    // Try to get from cache first
    if let Some(cached_username) = CACHED_GIT_USERNAME.get() {
        return Ok(cached_username.clone());
    }

    // If not cached, fetch from git
    let output = Command::new("git").arg("config").arg("user.name").output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if username.is_empty() {
                    Err(GitError::UnableToFetchGitUsername(
                        "Git username not configured in your system.".to_string(),
                        Vec::new(),
                    ))
                } else {
                    // Cache the username (this can only be set once)
                    let _ = CACHED_GIT_USERNAME.set(username.clone());
                    Ok(username)
                }
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                Err(GitError::UnableToFetchGitUsername(
                    error_message.to_string(),
                    Vec::new(),
                ))
            }
        }
        Err(e) => Err(GitError::UnableToFetchGitUsername(
            e.to_string(),
            Vec::new(),
        )),
    }
}

pub struct Commit {
    _hash: String,
    _time_since_epoch_ms: u64,
}

#[derive(Debug)]
pub enum GitError {
    UnableToFetchGitUsername(String, Vec<String>),
    PathError(utils::NormalisedPathError, Vec<String>),
}

impl error::WithBacktrace for GitError {
    fn backtrace(&self) -> &Vec<String> {
        match self {
            GitError::UnableToFetchGitUsername(_, s) => s,
            GitError::PathError(_, s) => s,
        }
    }

    fn add_context(&mut self, function_name: String) {
        match self {
            GitError::UnableToFetchGitUsername(_, s) => s.push(function_name),
            GitError::PathError(_, s) => s.push(function_name),
        }
    }
}

impl error::CustomErrorTrait for GitError {}

impl From<GitError> for Box<dyn error::CustomErrorTrait> {
    fn from(error: GitError) -> Self {
        Box::new(error)
    }
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::UnableToFetchGitUsername(e, _) => write!(f, "{}", e),
            GitError::PathError(e, _) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for GitError {}

impl From<utils::NormalisedPathError> for GitError {
    fn from(error: utils::NormalisedPathError) -> Self {
        GitError::PathError(error, Vec::new())
    }
}

#[track_errors]
pub fn get_commits_for_today_since_last_commit(
    _normalised_path: &utils::NormalisedGitPath,
    _last_commit: Option<Commit>,
) -> Result<Vec<Commit>, GitError> {
    let _username = get_git_username()?;

    // TODO: Then go to the path and run git commit, and get the ones from the username that are done today and after the last commit

    Ok(Vec::new())
}
