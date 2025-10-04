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
use crossterm::ExecutableCommand;
use ui::{draw_image_starting_at, pad_image_and_colours, print_in_box};
extern crate ctrlc;

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
    todo!();
}

#[async_trait]
impl CommandIfPetExists for RemovePetCommand {
    async fn execute(self, _pet: Pet, _user: UserInfo, _config: &mut Config) -> CommandResult {
        todo!();
    }
}

#[async_trait]
impl CommandIfPetExists for StatusCommand {
    async fn execute(self, pet: Pet, _user: UserInfo, _config: &mut Config) -> CommandResult {
        println!("{}", pet);
        do_status_animation(&pet).await
    }
}

async fn do_status_animation(pet: &Pet) -> CommandResult {
    print_in_box(
        |stdout, curr_cursor_y, box_width, box_height, _curr_frame| {
            // TODO: Need to add animation

            let ear_colour = "#0000ff";
            let eye_colour = match pet.happiness {
                h if h < 20.0 => "#ff0000", // red
                h if h < 70.0 => "#0000ff", // blue
                _ => "#00ff00",             // green
            };
            let mut eyes = match pet.happiness {
                h if h < 20.0 => "x.x",
                h if h < 70.0 => "o.o",
                _ => "^.^",
            };

            eyes = if _curr_frame % 40 == 0 && _curr_frame != 0 {
                "-.-"
            } else {
                eyes
            };

            let tongue_colour = "#ff0000";
            let tongue = match pet.hunger {
                h if h < 20.0 => "U",
                h if h < 70.0 => "-",
                _ => "~",
            };

            let full_face = [
                format!("/\\_/\\"),
                format!("( {} )", eyes),
                format!("=  {}  =", tongue),
            ]
            .join("\n");

            let colours = vec![
                vec![
                    ear_colour.to_string(),
                    ear_colour.to_string(),
                    ear_colour.to_string(),
                    ear_colour.to_string(),
                    "".to_string(),
                ],
                vec![
                    "".to_string(),
                    "".to_string(),
                    eye_colour.to_string(),
                    "".to_string(),
                    eye_colour.to_string(),
                ],
                vec![
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    tongue_colour.to_string(),
                ],
            ];

            // Pad lines to max width for alignment
            let (padded_face, padded_colours, max_width, max_height) =
                pad_image_and_colours(full_face, colours, None, None);

            // Position to draw face
            let start_x = box_width / 2 - max_width as u16 / 2;
            let start_y = curr_cursor_y + box_height / 2 - max_height as u16 / 2;

            draw_image_starting_at(stdout, &padded_face, &padded_colours, start_x, start_y)
        },
        150,
        Some(30),
    )
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

    todo!();
}

#[async_trait]
impl CommandIfPetExists for PlayCommand {
    async fn execute(self, _pet: Pet, _user: UserInfo, _config: &mut Config) -> CommandResult {
        todo!();
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
    tokio::task::spawn(async {
        ctrlc::set_handler(|| {
            final_cleanup();
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

    final_cleanup();
}

fn final_cleanup() {
    let mut stdout = std::io::stdout();
    stdout.execute(crossterm::cursor::Show).unwrap();
}
