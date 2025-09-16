use clap::Parser;
use colored::*;

mod commands;
mod config;

use commands::Commands;
use config::{Config, UserInfo};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

/// Result type for command operations
type CommandResult = Result<(), Box<dyn std::error::Error>>;

/// Helper function to check if user is authenticated
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
fn execute_authenticated_command<F>(config: &mut Config, operation: F) -> CommandResult
where
    F: FnOnce(UserInfo, &mut Config) -> CommandResult,
{
    let user = require_auth(&config)?;
    operation(user, config)
}

// Command handlers
fn handle_login(config: &Config) -> CommandResult {
    if let Some(user) = &config.user {
        println!("You are already logged in with email: {}", user.email);
    } else {
        // TODO: Implement login logic
        println!("Login functionality not yet implemented");
    }
    Ok(())
}

fn handle_logout(user: UserInfo, config: &mut Config) -> CommandResult {
    println!("Logging out user with email: {}", user.email);
    config.user = None;
    config.save()?;
    Ok(())
}

fn handle_whoami(user: UserInfo, _config: &mut Config) -> CommandResult {
    println!("Email: {}", user.email);
    Ok(())
}

fn handle_new_pet(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement new pet logic
    println!("New pet functionality not yet implemented");
    Ok(())
}

fn handle_remove_pet(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement remove pet logic
    println!("Remove pet functionality not yet implemented");
    Ok(())
}

fn handle_status(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement status logic
    println!("Status functionality not yet implemented");
    Ok(())
}

fn handle_feed(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement feed logic
    println!("Feed functionality not yet implemented");
    Ok(())
}

fn handle_play(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement play logic
    println!("Play functionality not yet implemented");
    Ok(())
}

fn handle_add_repo(_user: UserInfo, _config: &mut Config, path: &str) -> CommandResult {
    // TODO: Implement add repo logic
    println!(
        "Add repo functionality not yet implemented for path: {}",
        path
    );
    Ok(())
}

fn handle_remove_repo(_user: UserInfo, _config: &mut Config, path: &str) -> CommandResult {
    // TODO: Implement remove repo logic
    println!(
        "Remove repo functionality not yet implemented for path: {}",
        path
    );
    Ok(())
}

fn handle_list_repos(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement list repos logic
    println!("List repos functionality not yet implemented");
    Ok(())
}

fn main() {
    let args = Args::parse();

    // Load config at startup
    let mut config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", format!("Error loading config: {}", e).red());
            std::process::exit(1);
        }
    };

    let result = match args.command {
        Commands::Login {} => handle_login(&config),
        Commands::Logout {} => {
            execute_authenticated_command(&mut config, |u, c| handle_logout(u, c))
        }
        Commands::Whoami {} => {
            execute_authenticated_command(&mut config, |u, c| handle_whoami(u, c))
        }
        Commands::NewPet {} => {
            execute_authenticated_command(&mut config, |u, c| handle_new_pet(u, c))
        }
        Commands::RemovePet {} => {
            execute_authenticated_command(&mut config, |u, c| handle_remove_pet(u, c))
        }
        Commands::Status {} => {
            execute_authenticated_command(&mut config, |u, c| handle_status(u, c))
        }
        Commands::Feed {} => execute_authenticated_command(&mut config, |u, c| handle_feed(u, c)),
        Commands::Play {} => execute_authenticated_command(&mut config, |u, c| handle_play(u, c)),
        Commands::AddRepo { path } => {
            execute_authenticated_command(&mut config, |u, c| handle_add_repo(u, c, &path))
        }
        Commands::RemoveRepo { path } => {
            execute_authenticated_command(&mut config, |u, c| handle_remove_repo(u, c, &path))
        }
        Commands::ListRepos {} => {
            execute_authenticated_command(&mut config, |u, c| handle_list_repos(u, c))
        }
    };

    // Handle any errors from config operations
    if let Err(e) = result {
        eprintln!("{}", format!("Error: {}", e).red());
        std::process::exit(1);
    }
}
