use clap::Parser;

mod auth;
mod commands;
mod config;
mod constants;
mod error;
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
type CommandResult = Result<(), Box<dyn error::CustomErrorTrait>>;

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
        do_logout_impl(user, config).await
    }
}

async fn do_logout_impl(user: UserInfo, config: &mut Config) -> CommandResult {
    do_logout(user, config).await
}

#[async_trait]
impl AuthenticatedCommand for WhoamiCommand {
    async fn execute(self, user: UserInfo, _config: &mut Config) -> CommandResult {
        do_whoami_impl(user, _config).await
    }
}

async fn do_whoami_impl(user: UserInfo, _config: &mut Config) -> CommandResult {
    println!("Email: {}", user.email);
    println!("Username: {}", user.username);
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for NewPetCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        do_new_pet_impl(_user, _config).await
    }
}

async fn do_new_pet_impl(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement new pet logic
    println!("New pet functionality not yet implemented");
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for RemovePetCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        do_remove_pet_impl(_user, _config).await
    }
}

async fn do_remove_pet_impl(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement remove pet logic
    println!("Remove pet functionality not yet implemented");
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for StatusCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        do_status_impl(_user, _config).await
    }
}

async fn do_status_impl(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement status logic
    println!("Status functionality not yet implemented");
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for FeedCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        feed_impl(_user, config).await
    }
}

async fn feed_impl(_user: UserInfo, config: &mut Config) -> CommandResult {
    let normalised_paths = config.get_valid_normalised_paths_and_save()?;
    if normalised_paths.is_empty() {
        println!("No Git repositories added yet!");
        return Ok(());
    }

    for repo in normalised_paths {
        let _ = git::get_commits_for_today_since_last_commit(&repo, None)?;
    }

    // TODO: Implement feed logic
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for PlayCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        play_impl(_user, _config).await
    }
}

async fn play_impl(_user: UserInfo, _config: &mut Config) -> CommandResult {
    // TODO: Implement play logic
    println!("Play functionality not yet implemented");
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for AddRepoCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        add_repo_impl(self.path, config).await
    }
}

async fn add_repo_impl(path: String, config: &mut Config) -> CommandResult {
    let normalised_path = utils::NormalisedGitPath::new(path)?;

    if config.repos.contains(&normalised_path.to_string()) {
        return Err(format!("Repo already added: {}", normalised_path).into());
    }

    config.repos.push(normalised_path.to_string());
    config.save()?;

    println!("Added new Git repository successfully!");
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for RemoveRepoCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        remove_repo_impl(self.path, config).await
    }
}

async fn remove_repo_impl(path: String, config: &mut Config) -> CommandResult {
    let repo_path = match utils::NormalisedGitPath::new(path) {
        Ok(normalised_path) => normalised_path.to_string(),
        Err(utils::NormalisedPathError::PathNotGitRepository(path, _)) => path,
        Err(e) => return Err(e.into()),
    };

    if !config.repos.contains(&repo_path) {
        println!("Repository was never registered with BitPet, so nothing to remove!");
        return Ok(());
    }

    if config.repos.contains(&repo_path) {
        config
            .repos
            .remove(config.repos.iter().position(|r| r == &repo_path).unwrap());
        config.save()?;
    }
    println!("Removed repository successfully!");
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for ListReposCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        list_repos_impl(config).await
    }
}

async fn list_repos_impl(config: &mut Config) -> CommandResult {
    let normalised_paths = config.get_valid_normalised_paths_and_save()?;

    if normalised_paths.is_empty() {
        println!("No Git repositories added yet");
        return Ok(());
    }

    for normalised_path in normalised_paths {
        println!("- {}", normalised_path);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    let args = Args::parse();

    // Load config at startup
    let mut config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            utils::print_error_chain(e.into());
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
        utils::print_error_chain(e);
        std::process::exit(1);
    }
}
