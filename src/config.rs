use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::utils;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Config {
    /// User authentication information
    pub user: Option<UserInfo>,
    pub repos: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub token: String,
}

impl Config {
    /// Get the path to the config file
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir()
            .ok_or(ConfigError::NoConfigDir)?
            .join("bitpet");

        // Ensure the directory exists
        fs::create_dir_all(&config_dir).map_err(|e| ConfigError::IoError(e))?;

        Ok(config_dir.join("config.json"))
    }

    /// Load config from file, creating a default one if it doesn't exist
    pub fn load() -> Result<Config, ConfigError> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Create default config and save it
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path).map_err(|e| ConfigError::IoError(e))?;

        let config: Config =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e))?;

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::config_path()?;

        let content =
            serde_json::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e))?;

        fs::write(&config_path, content).map_err(|e| ConfigError::IoError(e))?;

        Ok(())
    }

    pub fn get_valid_normalised_paths_and_save(
        &mut self,
    ) -> Result<Vec<utils::NormalisedGitPath>, ConfigError> {
        let mut valid_repos = Vec::new();
        let mut valid_paths = Vec::new();

        for repo in &self.repos {
            match utils::NormalisedGitPath::new(repo.clone()) {
                Ok(normalised_path) => {
                    valid_repos.push(repo.clone());
                    valid_paths.push(normalised_path);
                }
                Err(_) => {
                    // Skip invalid repositories
                }
            }
        }

        self.repos = valid_repos;
        self.save()?;

        Ok(valid_paths)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    NoConfigDir,
    IoError(std::io::Error),
    ParseError(serde_json::Error),
    SerializeError(serde_json::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NoConfigDir => write!(f, "Could not determine config directory"),
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Failed to parse config: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Failed to serialize config: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
