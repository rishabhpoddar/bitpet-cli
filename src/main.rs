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
        Commands::Login {} => {
            println!("Logging in...");
            Ok(())
        }
        Commands::Logout {} => {
            println!("Logging out...");
            config.user = None;
            config.save()
        }
        Commands::Whoami {} => {
            if let Some(user) = &config.user {
                println!("Email: {}", user.email);
            } else {
                println!("Not logged in");
            }
            Ok(())
        }
        Commands::NewPet {} => {
            println!("Adopting a new pet...");
            Ok(())
        }
        Commands::RemovePet {} => {
            println!("Letting go of your pet...");
            Ok(())
        }
        Commands::Status {} => {
            println!("Getting status of your pet...");
            Ok(())
        }
        Commands::Feed {} => {
            println!("Feeding your pet...");
            Ok(())
        }
        Commands::Play {} => {
            println!("Playing with your pet...");
            Ok(())
        }
        Commands::AddRepo { path } => {
            println!("Adding a git repo: {}", path);
            Ok(())
        }
        Commands::RemoveRepo { path } => {
            println!("Removing a git repo: {}", path);
            Ok(())
        }
        Commands::ListRepos {} => {
            println!("Listing all git repos...");
            Ok(())
        }
    };

    // Handle any errors from config operations
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
