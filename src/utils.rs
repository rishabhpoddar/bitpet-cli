use crate::error;
use crate::error::WithBacktrace;
use crate::git;
use bitpet_cli::track_errors;
use colored::*;
use std::env;

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
    PathNotExists(String, Vec<String>),
    PathNotGitRepository(String, Vec<String>),
    Other(Box<dyn std::error::Error>, Vec<String>),
}

impl error::WithBacktrace for NormalisedPathError {
    fn backtrace(&self) -> &Vec<String> {
        match self {
            NormalisedPathError::PathNotExists(_, s)
            | NormalisedPathError::PathNotGitRepository(_, s)
            | NormalisedPathError::Other(_, s) => s,
        }
    }

    fn add_context(&mut self, function_name: String) {
        match self {
            NormalisedPathError::PathNotExists(_, s)
            | NormalisedPathError::PathNotGitRepository(_, s)
            | NormalisedPathError::Other(_, s) => s.push(function_name),
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
    #[track_errors]
    pub fn new(path: String) -> Result<NormalisedGitPath, NormalisedPathError> {
        if path.is_empty() {
            return Err(NormalisedPathError::PathNotExists(path, Vec::new()));
        }
        let path = if std::path::Path::new(&path).is_absolute() {
            std::path::PathBuf::from(path)
        } else {
            env::current_dir()
                .map_err(|e| NormalisedPathError::Other(e.into(), Vec::new()))?
                .join(path)
        };

        if !path.exists() {
            return Err(NormalisedPathError::PathNotExists(
                path.display().to_string(),
                Vec::new(),
            ));
        }

        let normalised_path = NormalisedGitPath {
            path: path
                .canonicalize()
                .map_err(|e| NormalisedPathError::Other(e.into(), Vec::new()))?,
        };

        if !git::is_git(&normalised_path) {
            return Err(NormalisedPathError::PathNotGitRepository(
                normalised_path.path.display().to_string(),
                Vec::new(),
            ));
        }

        Ok(normalised_path)
    }

    #[track_errors]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

/// Print an error and its full chain of causes
#[track_errors]
pub fn print_error_chain(error: Box<dyn error::CustomErrorTrait>) {
    eprintln!("{}", format!("Error: {}", error).red());

    let backtrace = error.backtrace();
    if !backtrace.is_empty() {
        eprintln!("{}", "Call stack:".cyan());
        for (i, func_name) in backtrace.iter().enumerate() {
            eprintln!("  {}: {}", i + 1, func_name.cyan());
        }
    }
}
