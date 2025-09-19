use crate::git;
use colored::*;
use std::env;
use std::error::Error;

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
    PathNotExists(String),
    PathNotGitRepository(String),
    Other(Box<dyn std::error::Error>),
}

impl std::fmt::Display for NormalisedPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NormalisedPathError::PathNotExists(path) => write!(f, "Path does not exist: {}", path),
            NormalisedPathError::PathNotGitRepository(path) => {
                write!(f, "Provided path is not a Git repository: {}", path)
            }
            NormalisedPathError::Other(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for NormalisedPathError {}

impl NormalisedGitPath {
    // NOTE: These are blocking function calls and are being called in an async context. But it is
    // OK cause this is client code anyway.
    pub fn new(path: String) -> Result<NormalisedGitPath, NormalisedPathError> {
        if path.is_empty() {
            return Err(NormalisedPathError::PathNotExists(path));
        }
        let path = if std::path::Path::new(&path).is_absolute() {
            std::path::PathBuf::from(path)
        } else {
            env::current_dir()
                .map_err(|e| NormalisedPathError::Other(e.into()))?
                .join(path)
        };

        if !path.exists() {
            return Err(NormalisedPathError::PathNotExists(
                path.display().to_string(),
            ));
        }

        let normalised_path = NormalisedGitPath {
            path: path
                .canonicalize()
                .map_err(|e| NormalisedPathError::Other(e.into()))?,
        };

        if !git::is_git(&normalised_path) {
            return Err(NormalisedPathError::PathNotGitRepository(
                normalised_path.path.display().to_string(),
            ));
        }

        Ok(normalised_path)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

/// Print an error and its full chain of causes
pub fn print_error_chain(error: &dyn Error) {
    eprintln!("{}", format!("Error: {}", error).red());

    let mut source = error.source();
    let mut indent = 1;
    while let Some(err) = source {
        eprintln!(
            "{}{}",
            "  ".repeat(indent),
            format!("Caused by: {}", err).yellow()
        );
        source = err.source();
        indent += 1;
    }
}
