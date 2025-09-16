use clap::Parser;

mod commands;
mod config;

use commands::Commands;
use config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

/// Result type for command operations
type CommandResult = Result<(), Box<dyn std::error::Error>>;

/// Helper function to check if user is authenticated
fn require_auth(config: &Config) -> Result<(), AuthError> {
    if config.user.is_some() {
        Ok(())
    } else {
        Err(AuthError::NotLoggedIn)
    }
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
    F: FnOnce(&mut Config) -> CommandResult,
{
    match require_auth(config) {
        Ok(()) => operation(config),
        Err(e) => Err(Box::new(e)),
    }
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

fn handle_logout(config: &mut Config) -> CommandResult {
    if config.user.is_some() {
        println!("Logging out...");
        config.user = None;
        config
            .save()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    } else {
        println!("Not logged in");
        Ok(())
    }
}

fn handle_whoami(config: &Config) -> CommandResult {
    // TODO: Make the type better of the input cause we are sure that the user is authenticated
    println!("Email: {}", config.user.as_ref().unwrap().email);
    Ok(())
}

fn handle_new_pet(_config: &mut Config) -> CommandResult {
    // TODO: Implement new pet logic
    println!("New pet functionality not yet implemented");
    Ok(())
}

fn handle_remove_pet(_config: &mut Config) -> CommandResult {
    // TODO: Implement remove pet logic
    println!("Remove pet functionality not yet implemented");
    Ok(())
}

fn handle_status(_config: &mut Config) -> CommandResult {
    // TODO: Implement status logic
    println!("Status functionality not yet implemented");
    Ok(())
}

fn handle_feed(_config: &mut Config) -> CommandResult {
    // TODO: Implement feed logic
    println!("Feed functionality not yet implemented");
    Ok(())
}

fn handle_play(_config: &mut Config) -> CommandResult {
    // TODO: Implement play logic
    println!("Play functionality not yet implemented");
    Ok(())
}

fn handle_add_repo(_config: &mut Config, path: &str) -> CommandResult {
    // TODO: Implement add repo logic
    println!(
        "Add repo functionality not yet implemented for path: {}",
        path
    );
    Ok(())
}

fn handle_remove_repo(_config: &mut Config, path: &str) -> CommandResult {
    // TODO: Implement remove repo logic
    println!(
        "Remove repo functionality not yet implemented for path: {}",
        path
    );
    Ok(())
}

fn handle_list_repos(_config: &mut Config) -> CommandResult {
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
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    let result = match args.command {
        Commands::Login {} => handle_login(&config),
        Commands::Logout {} => execute_authenticated_command(&mut config, |c| handle_logout(c)),
        Commands::Whoami {} => execute_authenticated_command(&mut config, |c| handle_whoami(c)),
        Commands::NewPet {} => execute_authenticated_command(&mut config, |c| handle_new_pet(c)),
        Commands::RemovePet {} => {
            execute_authenticated_command(&mut config, |c| handle_remove_pet(c))
        }
        Commands::Status {} => execute_authenticated_command(&mut config, |c| handle_status(c)),
        Commands::Feed {} => execute_authenticated_command(&mut config, |c| handle_feed(c)),
        Commands::Play {} => execute_authenticated_command(&mut config, |c| handle_play(c)),
        Commands::AddRepo { path } => {
            execute_authenticated_command(&mut config, |c| handle_add_repo(c, &path))
        }
        Commands::RemoveRepo { path } => {
            execute_authenticated_command(&mut config, |c| handle_remove_repo(c, &path))
        }
        Commands::ListRepos {} => {
            execute_authenticated_command(&mut config, |c| handle_list_repos(c))
        }
    };

    // Handle any errors from config operations
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

// TODO: Make eprintln print in red
