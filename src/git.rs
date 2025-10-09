use crate::utils;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::OnceLock;

use crate::error;

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

// Thread-safe, lazy-initialized static cache for git username
static CACHED_GIT_USERNAME: OnceLock<String> = OnceLock::new();

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
                        std::backtrace::Backtrace::capture().to_string(),
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
                    std::backtrace::Backtrace::capture().to_string(),
                ))
            }
        }
        Err(e) => Err(GitError::UnableToFetchGitUsername(
            e.to_string(),
            std::backtrace::Backtrace::capture().to_string(),
        )),
    }
}

// Thread-safe, lazy-initialized static cache for git username
static CACHED_GIT_EMAIL: OnceLock<String> = OnceLock::new();

fn get_git_email() -> Result<String, GitError> {
    // Try to get from cache first
    if let Some(cached_email) = CACHED_GIT_EMAIL.get() {
        return Ok(cached_email.clone());
    }

    // If not cached, fetch from git
    let output = Command::new("git").arg("config").arg("user.email").output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let email = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if email.is_empty() {
                    Err(GitError::UnableToFetchGitEmail(
                        "Git email not configured in your system.".to_string(),
                        std::backtrace::Backtrace::capture().to_string(),
                    ))
                } else {
                    // Cache the email (this can only be set once)
                    let _ = CACHED_GIT_EMAIL.set(email.clone());
                    Ok(email)
                }
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                Err(GitError::UnableToFetchGitEmail(
                    error_message.to_string(),
                    std::backtrace::Backtrace::capture().to_string(),
                ))
            }
        }
        Err(e) => Err(GitError::UnableToFetchGitEmail(
            e.to_string(),
            std::backtrace::Backtrace::capture().to_string(),
        )),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    hash: String,
    time_since_epoch_ms: u64,
}

impl std::fmt::Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Commit: {}\nTime since epoch: {}",
            self.hash, self.time_since_epoch_ms
        )
    }
}

#[derive(Debug)]
pub enum GitError {
    UnableToFetchGitUsername(String, String),
    UnableToFetchGitEmail(String, String),
    PathError(utils::NormalisedPathError, String),
    GitLogError(String, String),
}

impl error::WithBacktrace for GitError {
    fn backtrace(&self) -> &String {
        match self {
            GitError::UnableToFetchGitUsername(_, s) => s,
            GitError::UnableToFetchGitEmail(_, s) => s,
            GitError::PathError(_, s) => s,
            GitError::GitLogError(_, s) => s,
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
            GitError::UnableToFetchGitEmail(e, _) => write!(f, "{}", e),
            GitError::PathError(e, _) => write!(f, "{}", e),
            GitError::GitLogError(e, _) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for GitError {}

impl From<utils::NormalisedPathError> for GitError {
    fn from(error: utils::NormalisedPathError) -> Self {
        GitError::PathError(error, std::backtrace::Backtrace::capture().to_string())
    }
}

pub fn get_commits_for_path_since(
    normalised_path: &utils::NormalisedGitPath,
    since: &str,
) -> Result<Vec<Commit>, GitError> {
    let username = get_git_username()?;
    let email = get_git_email()?;

    let git_log_output = Command::new("git")
        .arg("log")
        .arg(format!("--since={}", since))
        .current_dir(normalised_path.path())
        .output();

    match git_log_output {
        Ok(output) => {
            if output.status.success() {
                let commits_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if commits_text.is_empty() {
                    return Ok(Vec::new());
                }

                let mut commits = Vec::new();
                let commit_blocks: Vec<&str> = commits_text.split("\ncommit ").collect();

                for (i, block) in commit_blocks.iter().enumerate() {
                    let block = if i == 0 {
                        block.strip_prefix("commit ").unwrap_or(block)
                    } else {
                        block
                    };

                    if let Some(commit) = parse_commit_block(block, &username, &email)? {
                        commits.push(commit);
                    }
                }

                Ok(commits)
            } else {
                let error_message = String::from_utf8_lossy(&output.stderr);
                Err(GitError::GitLogError(
                    error_message.to_string(),
                    std::backtrace::Backtrace::capture().to_string(),
                ))
            }
        }
        Err(e) => Err(GitError::GitLogError(
            e.to_string(),
            std::backtrace::Backtrace::capture().to_string(),
        )),
    }
}

fn parse_commit_block(
    block: &str,
    expected_username: &str,
    expected_email: &str,
) -> Result<Option<Commit>, GitError> {
    let lines: Vec<&str> = block.lines().collect();
    if lines.len() < 3 {
        return Ok(None);
    }

    // Parse commit hash (first line)
    let hash = lines[0].trim().to_string();
    if hash.is_empty() {
        return Ok(None);
    }

    // Parse author line (second line should be "Author: name <email>")
    let author_line = lines[1];
    if !author_line.starts_with("Author: ") {
        return Ok(None);
    }

    let author_info = &author_line[8..]; // Skip "Author: "

    // Extract author name and email from "name <email>" format
    let (author_name, author_email) = if let Some(email_start) = author_info.find(" <") {
        let name = author_info[..email_start].trim();
        let email_part = &author_info[email_start + 2..]; // Skip " <"
        let email = if let Some(email_end) = email_part.find('>') {
            email_part[..email_end].trim()
        } else {
            email_part.trim()
        };
        (name, email)
    } else {
        (author_info.trim(), "")
    };

    // Check if this commit is from the expected username OR email
    if author_name != expected_username && author_email != expected_email {
        return Ok(None);
    }

    // Parse date line (third line should be "Date:   ...")
    let date_line = lines[2];
    if !date_line.starts_with("Date:") {
        return Ok(None);
    }

    let date_str = date_line[5..].trim(); // Skip "Date:"

    // Parse the date string to get timestamp
    // Format: "Fri Sep 19 20:12:42 2025 +0530"
    let timestamp_ms = parse_git_date(date_str)?;

    Ok(Some(Commit {
        hash,
        time_since_epoch_ms: timestamp_ms,
    }))
}

fn parse_git_date(date_str: &str) -> Result<u64, GitError> {
    // Git date format is: "Day Mon DD HH:MM:SS YYYY +ZZZZ"
    let parsed_date =
        DateTime::parse_from_str(date_str, "%a %b %d %H:%M:%S %Y %z").map_err(|e| {
            GitError::GitLogError(
                format!("Failed to parse git date '{}': {}", date_str, e),
                std::backtrace::Backtrace::capture().to_string(),
            )
        })?;

    // Convert to milliseconds since Unix epoch
    Ok(parsed_date.timestamp_millis() as u64)
}
