use crate::error;

use crate::git;

use colored::*;
use std::env;

fn get_git_root_path(normalised_path: NormalisedGitPath) -> NormalisedGitPath {
    assert!(git::is_git(&normalised_path));
    let mut path = normalised_path.path();
    if path.join(".git").exists() {
        return NormalisedGitPath {
            path: path.to_path_buf(),
        };
    }
    while let Some(parent) = path.parent() {
        if parent.join(".git").exists() {
            return get_git_root_path(NormalisedGitPath {
                path: parent.to_path_buf(),
            });
        }
        path = parent;
    }
    let result = NormalisedGitPath {
        path: path.to_path_buf(),
    };
    assert!(git::is_git(&result));
    result
}

#[derive(Debug)]
pub struct NormalisedGitPath {
    path: std::path::PathBuf,
}

impl std::fmt::Display for NormalisedGitPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

#[derive(Debug)]
pub enum NormalisedPathError {
    PathNotExists(String, String),
    PathNotGitRepository(String, String),
    Other(Box<dyn std::error::Error>, String),
}

impl error::WithBacktrace for NormalisedPathError {
    fn backtrace(&self) -> &String {
        match self {
            NormalisedPathError::PathNotExists(_, s)
            | NormalisedPathError::PathNotGitRepository(_, s)
            | NormalisedPathError::Other(_, s) => s,
        }
    }
}

impl error::CustomErrorTrait for NormalisedPathError {}

impl From<NormalisedPathError> for Box<dyn error::CustomErrorTrait> {
    fn from(error: NormalisedPathError) -> Self {
        Box::new(error)
    }
}

impl std::fmt::Display for NormalisedPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NormalisedPathError::PathNotExists(path, _) => {
                write!(f, "Path does not exist: {}", path)
            }
            NormalisedPathError::PathNotGitRepository(path, _) => {
                write!(f, "Provided path is not a Git repository: {}", path)
            }
            NormalisedPathError::Other(error, _) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for NormalisedPathError {}

impl NormalisedGitPath {
    // NOTE: These are blocking function calls and are being called in an async context. But it is
    // OK cause this is client code anyway.

    pub fn new(path: String) -> Result<NormalisedGitPath, NormalisedPathError> {
        if path.is_empty() {
            return Err(NormalisedPathError::PathNotExists(
                path,
                std::backtrace::Backtrace::capture().to_string(),
            ));
        }
        let path = if std::path::Path::new(&path).is_absolute() {
            std::path::PathBuf::from(path)
        } else {
            env::current_dir()
                .map_err(|e| {
                    NormalisedPathError::Other(
                        e.into(),
                        std::backtrace::Backtrace::capture().to_string(),
                    )
                })?
                .join(path)
        };

        if !path.exists() {
            return Err(NormalisedPathError::PathNotExists(
                path.display().to_string(),
                std::backtrace::Backtrace::capture().to_string(),
            ));
        }

        let normalised_path = NormalisedGitPath {
            path: path.canonicalize().map_err(|e| {
                NormalisedPathError::Other(
                    e.into(),
                    std::backtrace::Backtrace::capture().to_string(),
                )
            })?,
        };

        if !git::is_git(&normalised_path) {
            return Err(NormalisedPathError::PathNotGitRepository(
                normalised_path.path.display().to_string(),
                std::backtrace::Backtrace::capture().to_string(),
            ));
        }

        let root_path = get_git_root_path(normalised_path);

        Ok(root_path)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

/// Print an error and its full chain of causes

pub fn print_error_chain(error: Box<dyn error::CustomErrorTrait>) {
    eprintln!("{}", format!("Error: {}", error).red());

    let backtrace = error.backtrace();
    if !backtrace.is_empty() {
        eprintln!("{}", backtrace.cyan().dimmed());
    }
}
