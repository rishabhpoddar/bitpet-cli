use crate::utils;
use std::process::Command;

// NOTE: These are blocking function calls and are being called in an async context. But it is
// OK cause this is client code anyway.
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

fn get_git_username() -> Result<String, String> {
    let output = Command::new("git").arg("config").arg("user.name").output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if username.is_empty() {
                    Err("Git username not configured in your system.".to_string())
                } else {
                    Ok(username)
                }
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                Err(format!("Git command failed: {}", error_message))
            }
        }
        Err(e) => Err(format!("Failed to execute Git command: {}", e)),
    }
}

pub struct Commit {
    _hash: String,
    _time_since_epoch_ms: u64,
}

#[derive(Debug)]
pub enum GitError {
    UnableToFetchGitUsername(String),
    PathError(utils::NormalisedPathError),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::UnableToFetchGitUsername(e) => write!(f, "{}", e),
            GitError::PathError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for GitError {}

impl From<utils::NormalisedPathError> for GitError {
    fn from(error: utils::NormalisedPathError) -> Self {
        GitError::PathError(error)
    }
}

pub fn get_commits_for_today_since_last_commit(
    _normalised_path: &utils::NormalisedGitPath,
    _last_commit: Option<Commit>,
) -> Result<Vec<Commit>, GitError> {
    let username = get_git_username();

    // TODO: Then go to the path and run git commit, and get the ones from the username that are done today and after the last commit

    match username {
        Ok(username) => {
            println!("Username: {}", username);
            let commits = Vec::new();
            Ok(commits)
        }
        Err(e) => Err(GitError::UnableToFetchGitUsername(e)),
    }
}
