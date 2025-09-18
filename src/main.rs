use clap::Parser;
use colored::*;

mod auth;
mod commands;
mod config;
mod constants;
mod git;
mod http_mocking;
mod utils;

use async_trait::async_trait;
use auth::{AuthenticatedCommand, do_login, do_logout, execute_authenticated_command};
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

// Command structs implementing AuthenticatedCommand trait
struct LogoutCommand;
struct WhoamiCommand;
struct NewPetCommand;
struct RemovePetCommand;
struct StatusCommand;
struct FeedCommand;
struct PlayCommand;
struct AddRepoCommand {
    path: String,
}
struct RemoveRepoCommand {
    path: String,
}
struct ListReposCommand;

// Command handlers
async fn handle_login(config: &mut Config) -> CommandResult {
    if let Some(_user) = &config.user {
        return Err(format!("You are already logged in with email: {}", _user.email).into());
    } else {
        do_login(config).await
    }
}

// Implement AuthenticatedCommand trait for all command structs
#[async_trait]
impl AuthenticatedCommand for LogoutCommand {
    async fn execute(self, user: UserInfo, config: &mut Config) -> CommandResult {
        do_logout(user, config).await
    }
}

#[async_trait]
impl AuthenticatedCommand for WhoamiCommand {
    async fn execute(self, user: UserInfo, _config: &mut Config) -> CommandResult {
        println!("Email: {}", user.email);
        println!("Username: {}", user.username);
        Ok(())
    }
}

#[async_trait]
impl AuthenticatedCommand for NewPetCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        // TODO: Implement new pet logic
        println!("New pet functionality not yet implemented");
        Ok(())
    }
}

#[async_trait]
impl AuthenticatedCommand for RemovePetCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        // TODO: Implement remove pet logic
        println!("Remove pet functionality not yet implemented");
        Ok(())
    }
}

#[async_trait]
impl AuthenticatedCommand for StatusCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        // TODO: Implement status logic
        println!("Status functionality not yet implemented");
        Ok(())
    }
}

#[async_trait]
impl AuthenticatedCommand for FeedCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        // TODO: Implement feed logic
        println!("Feed functionality not yet implemented");
        Ok(())
    }
}

#[async_trait]
impl AuthenticatedCommand for PlayCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        // TODO: Implement play logic
        println!("Play functionality not yet implemented");
        Ok(())
    }
}

#[async_trait]
impl AuthenticatedCommand for AddRepoCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        let normalised_path = utils::NormalisedPath::new(self.path)?;

        if !git::is_git(&normalised_path) {
            return Err(
                format!("Provided path is not a git repository: {}", normalised_path).into(),
            );
        }

        if config.repos.contains(&normalised_path.to_string()) {
            return Err(format!("Repo already added: {}", normalised_path).into());
        }

        config.repos.push(normalised_path.to_string());
        config.save()?;

        println!("Added new Git repository successfully!");
        Ok(())
    }
}

#[async_trait]
impl AuthenticatedCommand for RemoveRepoCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        let normalised_path = utils::NormalisedPath::new(self.path)?;

        if !config.repos.contains(&normalised_path.to_string()) {
            println!("Repository was never registered with BitPet, so nothing to remove!");
            return Ok(());
        }
        remove_repo(config, normalised_path.to_string())?;
        println!("Removed repository successfully!");
        Ok(())
    }
}

fn remove_repo(config: &mut Config, repo: String) -> Result<(), Box<dyn std::error::Error>> {
    if config.repos.contains(&repo) {
        config
            .repos
            .remove(config.repos.iter().position(|r| r == &repo).unwrap());
        config.save()?;
    }
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for ListReposCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        let mut repos_to_remove = Vec::new();

        if config.repos.is_empty() {
            println!("No Git repositories added yet");
            return Ok(());
        }

        for repo in &config.repos {
            let normalised_path = utils::NormalisedPath::new(repo.clone());
            if let Err(_e) = normalised_path {
                repos_to_remove.push(repo.clone());
                continue;
            }

            if !git::is_git(&normalised_path.unwrap()) {
                repos_to_remove.push(repo.clone());
                continue;
            }

            println!("{}", repo);
        }

        // Remove invalid repos after iteration
        for repo in repos_to_remove {
            remove_repo(config, repo)?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
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
        Commands::Login {} => handle_login(&mut config).await,
        Commands::Logout {} => execute_authenticated_command(&mut config, LogoutCommand).await,
        Commands::Whoami {} => execute_authenticated_command(&mut config, WhoamiCommand).await,
        Commands::NewPet {} => execute_authenticated_command(&mut config, NewPetCommand).await,
        Commands::RemovePet {} => {
            execute_authenticated_command(&mut config, RemovePetCommand).await
        }
        Commands::Status {} => execute_authenticated_command(&mut config, StatusCommand).await,
        Commands::Feed {} => execute_authenticated_command(&mut config, FeedCommand).await,
        Commands::Play {} => execute_authenticated_command(&mut config, PlayCommand).await,
        Commands::AddRepo { path } => {
            execute_authenticated_command(&mut config, AddRepoCommand { path }).await
        }
        Commands::RemoveRepo { path } => {
            execute_authenticated_command(&mut config, RemoveRepoCommand { path }).await
        }
        Commands::ListRepos {} => {
            execute_authenticated_command(&mut config, ListReposCommand).await
        }
    };

    // Handle any errors from config operations
    if let Err(e) = result {
        eprintln!("{}", format!("Error: {}", e).red());
        std::process::exit(1);
    }
}
