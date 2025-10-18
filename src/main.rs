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
use ui::{draw_animation_in_center_of_box, final_cleanup_for_terminal};
extern crate ctrlc;
extern crate reqwest;
extern crate reqwest_middleware;
use crate::http_mocking::MockingMiddleware;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use std::collections::HashMap;

use async_trait::async_trait;
use auth::{AuthenticatedCommand, do_login, do_logout, execute_authenticated_command};

use commands::Commands;
use config::{Config, UserInfo};
use constants::UPDATE_CHECK_PATH;
use pet::{
    CommandIfPetExists, execute_command_if_pet_exists, feed_pet, get_pet_status, play_with_pet,
    submit_challenge_answer,
};

use crate::pet::FeedStatus;

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
struct ChallengeReadCommand;
struct ChallengeAnswerCommand;
struct ChallengeRemoveCommand;

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
impl CommandIfPetExists for StatusCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        let pet = get_pet_status(_user.token.as_str(), _config).await?;
        println!("{}", pet.0);
        draw_animation_in_center_of_box(&pet.1).await
    }
}

#[async_trait]
impl CommandIfPetExists for FeedCommand {
    async fn execute(self, user: UserInfo, config: &mut Config) -> CommandResult {
        feed_impl(user, config).await
    }
}

async fn feed_impl(_user: UserInfo, config: &mut Config) -> CommandResult {
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

    let feed_result = feed_pet(_user.token.as_str(), config, commits).await?;

    config.challenge = None;
    config.save()?;

    match feed_result.status {
        FeedStatus::AskForChallenge => {
            println!("Your pet is asking for a coding challenge! Do you accept (Y/n)?");
            let mut accepted = false;
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim() == "Y" {
                    accepted = true;
                    break;
                } else if input.trim() == "n" {
                    break;
                }
                println!("Invalid input! Please enter Y or n");
            }
            if accepted {
                config.challenge = feed_result.challenge.clone();
                config.save()?;
                if let Some(text_before_animation) = feed_result.text_before_animation {
                    println!("{}", text_before_animation);
                }
                if let Some(animation) = feed_result.animation {
                    draw_animation_in_center_of_box(&animation).await?;
                }
                println!("{}", feed_result.challenge.unwrap());
                println!("Please answer the challenge by typing 'pet challenge ans'");
            } else {
                println!("You declined the challenge!");
            }
            Ok(())
        }
        _ => {
            if let Some(text_before_animation) = feed_result.text_before_animation {
                println!("{}", text_before_animation);
            }
            if let Some(animation) = feed_result.animation {
                draw_animation_in_center_of_box(&animation).await?;
            }
            if let Some(pet) = feed_result.pet {
                println!("{}", pet);
            }
            Ok(())
        }
    }
}

#[async_trait]
impl CommandIfPetExists for PlayCommand {
    async fn execute(self, _user: UserInfo, _config: &mut Config) -> CommandResult {
        let response = play_with_pet(_user.token.as_str(), _config).await?;
        if let Some(text_before_animation) = response.text_before_animation {
            println!("{}", text_before_animation);
        }
        if let Some(animation) = response.animation {
            draw_animation_in_center_of_box(&animation).await?;
        }
        if let Some(pet) = response.pet {
            println!("{}", pet);
        }
        Ok(())
    }
}

#[async_trait]
impl CommandIfPetExists for AddRepoCommand {
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
impl CommandIfPetExists for RemoveRepoCommand {
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
impl CommandIfPetExists for ListReposCommand {
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

#[async_trait]
impl AuthenticatedCommand for ChallengeReadCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        challenge_read_impl(_user, config).await
    }
}

async fn challenge_read_impl(_user: UserInfo, config: &mut Config) -> CommandResult {
    if let Some(challenge) = config.challenge.clone() {
        println!("{}", challenge);
    } else {
        println!(
            "\x1b[31mNo challenge found! Type 'pet feed' and you may get a new challenge!\x1b[0m"
        );
    }
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for ChallengeAnswerCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        challenge_answer_impl(_user, config).await
    }
}

async fn challenge_answer_impl(_user: UserInfo, config: &mut Config) -> CommandResult {
    if let Some(challenge) = config.challenge.clone() {
        println!("{}", challenge);
        let response = match challenge.answer_type {
            pet::ChallengeAnswerType::File => {
                println!("Please enter the path to the file you want to submit:");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let path = input.trim().to_string();
                let file_content = std::fs::read_to_string(path)?;
                submit_challenge_answer(_user.token.as_str(), config, challenge.id, file_content)
                    .await?
            }
            pet::ChallengeAnswerType::Text => {
                println!("Please enter the text you want to submit:");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let text = input.trim().to_string();
                submit_challenge_answer(_user.token.as_str(), config, challenge.id, text).await?
            }
        };

        match response.status {
            pet::ChallengeAnswerStatus::Correct => {
                let feed_result = response.feed_result.unwrap();
                if let Some(text_before_animation) = feed_result.text_before_animation {
                    println!("{}", text_before_animation);
                }
                if let Some(animation) = feed_result.animation {
                    draw_animation_in_center_of_box(&animation).await?;
                }
                if let Some(pet) = feed_result.pet {
                    println!("{}", pet);
                }
            }
            pet::ChallengeAnswerStatus::Incorrect => {
                println!("\x1b[31mIncorrect answer! Please try again!\x1b[0m");
            }
        }
    } else {
        println!(
            "\x1b[31mNo challenge found! Type 'pet feed' and you may get a new challenge!\x1b[0m"
        );
    }
    Ok(())
}

#[async_trait]
impl AuthenticatedCommand for ChallengeRemoveCommand {
    async fn execute(self, _user: UserInfo, config: &mut Config) -> CommandResult {
        challenge_remove_impl(_user, config).await
    }
}

async fn challenge_remove_impl(_user: UserInfo, config: &mut Config) -> CommandResult {
    if let Some(_) = config.challenge.clone() {
        config.challenge = None;
        config.save()?;
        println!("Removed challenge successfully!");
    } else {
        println!(
            "\x1b[31mNo challenge found! Type 'pet feed' and you may get a new challenge!\x1b[0m"
        );
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCheckAPIResult {
    pub update_available: bool,
}

async fn check_for_updates(token: Option<&str>) -> () {
    let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
        .with(MockingMiddleware)
        .build();
    let response = client
        .get("https://api.bitpet.dev".to_owned() + UPDATE_CHECK_PATH)
        .query(&[("curr_version", env!("CARGO_PKG_VERSION"))])
        .bearer_auth(token.unwrap_or(""))
        .send()
        .await;
    if response.is_ok() && response.as_ref().unwrap().status().is_success() {
        let api_result = response.unwrap().json::<UpdateCheckAPIResult>().await;
        if api_result.is_ok() {
            let api_result = api_result.unwrap();
            if api_result.update_available {
                println!(
                    "\x1b[33mIMPORTANT: A new version of BitPet is available! Please run 'TODO' to update.\x1b[0m"
                );
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tokio::task::spawn(async {
        ctrlc::set_handler(|| {
            let mut stdout = std::io::stdout();
            final_cleanup_for_terminal(&mut stdout);
            std::process::exit(1);
        })
        .unwrap();
    });

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
        Commands::Challenge { subcommand } => match subcommand {
            commands::ChallengeSubcommand::Read {} => {
                execute_authenticated_command(&mut config, ChallengeReadCommand).await
            }
            commands::ChallengeSubcommand::Ans {} => {
                execute_authenticated_command(&mut config, ChallengeAnswerCommand).await
            }
            commands::ChallengeSubcommand::Remove {} => {
                execute_authenticated_command(&mut config, ChallengeRemoveCommand).await
            }
        },
        Commands::Version {} => {
            println!("BitPet v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };

    // Handle any errors from config operations
    if let Err(e) = result {
        utils::print_error_chain(e);
        std::process::exit(1);
    }

    if config.last_update_check_time_ms + (1000 * 60 * 60 * 24)
        < std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    {
        let token: Option<&str> = match config.user {
            Some(ref user) => Some(&user.token),
            None => None,
        };
        check_for_updates(token).await;
        config.last_update_check_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let _ = config.save();
    }
}
