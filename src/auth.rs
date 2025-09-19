use crate::CommandResult;
use crate::config::{Config, UserInfo};
use crate::constants::{LOGIN_PATH, LOGOUT_PATH};
use crate::error;
use crate::http_mocking::MockingMiddleware;
use async_trait::async_trait;
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use std::iter;

fn require_auth(config: &Config) -> Result<UserInfo, AuthError> {
    config
        .user
        .as_ref()
        .cloned()
        .ok_or(AuthError::NotLoggedIn(Vec::new()))
}

#[derive(Debug)]
enum AuthError {
    NotLoggedIn(Vec<String>),
}

impl error::WithBacktrace for AuthError {
    fn backtrace(&self) -> &Vec<String> {
        match self {
            AuthError::NotLoggedIn(s) => s,
        }
    }

    fn add_context(&mut self, function_name: String) {
        match self {
            AuthError::NotLoggedIn(s) => s.push(function_name),
        }
    }
}

impl error::CustomErrorTrait for AuthError {}

impl From<AuthError> for Box<dyn error::CustomErrorTrait> {
    fn from(error: AuthError) -> Self {
        Box::new(error)
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::NotLoggedIn(_) => write!(f, "Please login first using 'pet login'"),
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

#[async_trait]
pub trait AuthenticatedCommand {
    async fn execute(self, user: UserInfo, config: &mut Config) -> CommandResult;
}
pub async fn execute_authenticated_command(
    config: &mut Config,
    command: impl AuthenticatedCommand,
) -> CommandResult {
    let user = require_auth(&config)?;
    command.execute(user, config).await
}

pub async fn do_logout(user: UserInfo, config: &mut Config) -> CommandResult {
    println!("Logging out user with email: {}", user.email);

    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .post("https://api.bitpet.dev".to_owned() + LOGOUT_PATH)
        .bearer_auth(user.token)
        .send()
        .await?;

    if response.status().is_success() || response.status().as_u16() == 401 {
        config.user = None;
        config.save()?;
        println!("Logged out successfully!");
        Ok(())
    } else {
        let error_text = response.text().await?;
        Err(format!("Logout failed: {}", error_text).into())
    }
}

pub async fn do_login(config: &mut Config) -> CommandResult {
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

    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .post("https://api.bitpet.dev".to_owned() + LOGIN_PATH)
        .body(serde_json::to_string(&json!({
            "url_code": random_string,
            "user_code": code
        }))?)
        .send()
        .await?;

    if response.status().is_success() {
        let login_response: LoginResponse = response.json().await?;

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
        let error_text = response.text().await?;
        Err(format!("Login failed: {}", error_text).into())
    }
}
