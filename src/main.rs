use clap::Parser;
use colored::*;

mod auth;
mod commands;
mod config;
mod constants;

use auth::{do_login, do_logout, execute_authenticated_command};
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

// Command handlers
fn handle_login(config: &mut Config) -> CommandResult {
    if let Some(_user) = &config.user {
        return Err(format!("You are already logged in with email: {}", _user.email).into());
    } else {
        do_login(config)
    }
}

fn handle_logout(user: UserInfo, config: &mut Config) -> CommandResult {
    do_logout(user, config)
}

fn handle_whoami(user: UserInfo, _config: &mut Config) -> CommandResult {
    println!("Email: {}", user.email);
    println!("Username: {}", user.username);
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
        Commands::Login {} => handle_login(&mut config),
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
