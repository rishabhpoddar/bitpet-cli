use crate::CommandResult;
use crate::config::{Config, UserInfo};

fn require_auth(config: &Config) -> Result<UserInfo, AuthError> {
    config.user.as_ref().cloned().ok_or(AuthError::NotLoggedIn)
}

#[derive(Debug)]
enum AuthError {
    NotLoggedIn,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::NotLoggedIn => write!(f, "Please login first using 'pet login'"),
        }
    }
}

impl std::error::Error for AuthError {}

/// Execute a command that requires authentication
pub fn execute_authenticated_command<F>(config: &mut Config, operation: F) -> CommandResult
where
    F: FnOnce(UserInfo, &mut Config) -> CommandResult,
{
    let user = require_auth(&config)?;
    operation(user, config)
}
