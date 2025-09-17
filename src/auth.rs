use crate::CommandResult;
use crate::config::{Config, UserInfo};
use crate::constants::{MOCK_USER_CODE, MOCK_USER_EMAIL, MOCK_USER_TOKEN, MOCK_USER_USERNAME};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use std::iter;

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

#[derive(Debug, Deserialize)]
struct LoginResponse {
    username: String,
    email: String,
    token: String,
}

/// Execute a command that requires authentication
pub fn execute_authenticated_command<F>(config: &mut Config, operation: F) -> CommandResult
where
    F: FnOnce(UserInfo, &mut Config) -> CommandResult,
{
    let user = require_auth(&config)?;
    operation(user, config)
}

pub fn do_logout(user: UserInfo, config: &mut Config) -> CommandResult {
    println!("Logging out user with email: {}", user.email);

    if user.email == MOCK_USER_EMAIL {
        config.user = None;
        config.save()?;
        println!("Logged out successfully!");
        return Ok(());
    }

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.bitpet.dev/v1/auth/logout")
        .bearer_auth(user.token)
        .send()?;

    if response.status().is_success() || response.status().as_u16() == 401 {
        config.user = None;
        config.save()?;
        println!("Logged out successfully!");
        Ok(())
    } else {
        let error_text = response.text()?;
        Err(format!("Logout failed: {}", error_text).into())
    }
}

pub fn do_login(config: &mut Config) -> CommandResult {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
    let random_string: String = iter::repeat_with(one_char).take(30).collect();

    println!(
        "Open the following URL in your browser: https://bitpet.dev/login?code={}",
        random_string
    );
    println!("\nOR\n\nPress Enter to auto-open in your browser...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim() == "" {
        open::that(format!("https://bitpet.dev/login?code={}", random_string)).unwrap();
    }

    print!("Once you login, you will see a code on your browser. Enter it here: ");
    std::io::stdout().flush().unwrap();
    let mut code = String::new();
    std::io::stdin().read_line(&mut code).unwrap();
    let code = code.trim();
    println!("\nLogging in...");

    if code == MOCK_USER_CODE {
        config.user = Some(UserInfo {
            username: MOCK_USER_USERNAME.to_string(),
            email: MOCK_USER_EMAIL.to_string(),
            token: MOCK_USER_TOKEN.to_string(),
        });
        config.save()?;

        println!("Successfully logged in as: {}", MOCK_USER_EMAIL);
        return Ok(());
    }

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.bitpet.dev/v1/auth/login")
        .json(&json!({
            "otp": random_string,
            "code": code
        }))
        .send()?;

    if response.status().is_success() {
        let login_response: LoginResponse = response.json()?;

        // Save user info to config
        config.user = Some(UserInfo {
            username: login_response.username.clone(),
            email: login_response.email.clone(),
            token: login_response.token,
        });
        config.save()?;

        println!("Successfully logged in as: {}", login_response.email);
        Ok(())
    } else {
        let error_text = response.text()?;
        Err(format!("Login failed: {}", error_text).into())
    }
}
