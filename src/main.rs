use clap::Parser;

mod auth;
mod commands;
mod config;
mod constants;
mod error;
mod git;
mod http_mocking;
mod pet;
mod ui;
mod utils;
use crossterm::QueueableCommand;
use crossterm::{ExecutableCommand, cursor, style::Print};
use std::io::{Write, stdout};
use std::time::Duration;
use tokio::time::sleep;

use sha2::{Digest, Sha256};

use std::collections::HashMap;

use async_trait::async_trait;
use auth::{AuthenticatedCommand, do_login, do_logout, execute_authenticated_command};

use commands::Commands;
use config::{Config, UserInfo};
use pet::{CommandIfPetExists, Pet, execute_command_if_pet_exists};

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
impl CommandIfPetExists for RemovePetCommand {
    async fn execute(self, _pet: Pet, _user: UserInfo, _config: &mut Config) -> CommandResult {
        // TODO: Implement remove pet logic
        println!("Remove pet functionality not yet implemented");
        Ok(())
    }
}

#[async_trait]
impl CommandIfPetExists for StatusCommand {
    async fn execute(self, pet: Pet, _user: UserInfo, _config: &mut Config) -> CommandResult {
        println!("{}", pet);
        // Instead of just dumping Debug
        do_status_animation().await?;
        Ok(())
    }
}

async fn do_status_animation() -> Result<(), std::io::Error> {
    let mut stdout = stdout();

    // Example "pet" frames
    let frames = vec![r"  (\_._/) ", r"  ( o.o ) ", r"  (> ^ <) "];

    // Save the cursor position before we start
    stdout.execute(cursor::SavePosition)?;

    for i in 0..10 {
        let frame = frames[i % frames.len()];

        // Jump back to saved cursor pos and overwrite only the animation area
        stdout.queue(cursor::RestorePosition)?;
        stdout.queue(Print(frame))?;
        stdout.flush()?;

        sleep(Duration::from_millis(300)).await;
    }

    // Move cursor down after animation so the prompt continues below
    stdout.execute(cursor::MoveToNextLine(2))?;
    Ok(())
}

#[async_trait]
impl CommandIfPetExists for FeedCommand {
    async fn execute(self, pet: Pet, user: UserInfo, config: &mut Config) -> CommandResult {
        feed_impl(pet, user, config).await
    }
}

async fn feed_impl(_pet: Pet, _user: UserInfo, config: &mut Config) -> CommandResult {
    let normalised_paths = config.get_valid_normalised_paths_and_save()?;
    if normalised_paths.is_empty() {
        println!("No Git repositories added yet!");
        return Ok(());
    }

    let mut commits: HashMap<String, Vec<git::Commit>> = HashMap::new();

    for repo in normalised_paths {
        let _commits = git::get_commits_for_path_since(&repo, "1week")?;
        commits.insert(
            format!("{:x}", Sha256::digest(repo.to_string().as_bytes())),
            _commits,
        );
    }

    // TODO: Implement feed logic
    Ok(())
}

#[async_trait]
impl CommandIfPetExists for PlayCommand {
    async fn execute(self, _pet: Pet, _user: UserInfo, _config: &mut Config) -> CommandResult {
        // TODO: Implement play logic
        println!("Play functionality not yet implemented");
        Ok(())
    }
}

#[async_trait]
impl CommandIfPetExists for AddRepoCommand {
    async fn execute(self, _pet: Pet, _user: UserInfo, config: &mut Config) -> CommandResult {
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
impl CommandIfPetExists for RemoveRepoCommand {
    async fn execute(self, _pet: Pet, _user: UserInfo, config: &mut Config) -> CommandResult {
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
impl CommandIfPetExists for ListReposCommand {
    async fn execute(self, _pet: Pet, _user: UserInfo, config: &mut Config) -> CommandResult {
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
            execute_command_if_pet_exists(&mut config, RemovePetCommand).await
        }
        Commands::Status {} => execute_command_if_pet_exists(&mut config, StatusCommand).await,
        Commands::Feed {} => execute_command_if_pet_exists(&mut config, FeedCommand).await,
        Commands::Play {} => execute_command_if_pet_exists(&mut config, PlayCommand).await,
        Commands::AddRepo { path } => {
            execute_command_if_pet_exists(&mut config, AddRepoCommand { path }).await
        }
        Commands::RemoveRepo { path } => {
            execute_command_if_pet_exists(&mut config, RemoveRepoCommand { path }).await
        }
        Commands::ListRepos {} => {
            execute_command_if_pet_exists(&mut config, ListReposCommand).await
        }
    };

    // Handle any errors from config operations
    if let Err(e) = result {
        utils::print_error_chain(e);
        std::process::exit(1);
    }
}
