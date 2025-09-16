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
fn require_auth(config: &Config) -> Result<(), &'static str> {
    if config.user.is_some() {
        Ok(())
    } else {
        Err("Please login first using 'pet login'")
    }
}

/// Execute a command that requires authentication
fn execute_authenticated_command<F>(config: &Config, operation: F) -> CommandResult
where
    F: FnOnce() -> CommandResult,
{
    match require_auth(config) {
        Ok(()) => operation(),
        Err(msg) => {
            println!("{}", msg);
            Ok(())
        }
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
    if let Some(user) = &config.user {
        println!("Email: {}", user.email);
    } else {
        println!("Not logged in");
    }
    Ok(())
}

fn handle_new_pet() -> CommandResult {
    // TODO: Implement new pet logic
    println!("New pet functionality not yet implemented");
    Ok(())
}

fn handle_remove_pet() -> CommandResult {
    // TODO: Implement remove pet logic
    println!("Remove pet functionality not yet implemented");
    Ok(())
}

fn handle_status() -> CommandResult {
    // TODO: Implement status logic
    println!("Status functionality not yet implemented");
    Ok(())
}

fn handle_feed() -> CommandResult {
    // TODO: Implement feed logic
    println!("Feed functionality not yet implemented");
    Ok(())
}

fn handle_play() -> CommandResult {
    // TODO: Implement play logic
    println!("Play functionality not yet implemented");
    Ok(())
}

fn handle_add_repo(path: &str) -> CommandResult {
    // TODO: Implement add repo logic
    println!(
        "Add repo functionality not yet implemented for path: {}",
        path
    );
    Ok(())
}

fn handle_remove_repo(path: &str) -> CommandResult {
    // TODO: Implement remove repo logic
    println!(
        "Remove repo functionality not yet implemented for path: {}",
        path
    );
    Ok(())
}

fn handle_list_repos() -> CommandResult {
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
        Commands::Logout {} => handle_logout(&mut config),
        Commands::Whoami {} => handle_whoami(&config),
        Commands::NewPet {} => execute_authenticated_command(&config, || handle_new_pet()),
        Commands::RemovePet {} => execute_authenticated_command(&config, || handle_remove_pet()),
        Commands::Status {} => execute_authenticated_command(&config, || handle_status()),
        Commands::Feed {} => execute_authenticated_command(&config, || handle_feed()),
        Commands::Play {} => execute_authenticated_command(&config, || handle_play()),
        Commands::AddRepo { path } => {
            execute_authenticated_command(&config, || handle_add_repo(&path))
        }
        Commands::RemoveRepo { path } => {
            execute_authenticated_command(&config, || handle_remove_repo(&path))
        }
        Commands::ListRepos {} => execute_authenticated_command(&config, || handle_list_repos()),
    };

    // Handle any errors from config operations
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
