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
            if let Some(user) = &config.user {
                println!("You are already logged in with email: {}", user.email);
                Ok(())
            } else {
                // TODO:...
                Ok(())
            }
        }
        Commands::Logout {} => {
            if let Some(_user) = &config.user {
                println!("Logging out...");
                config.user = None;
                config.save()
            } else {
                println!("Not logged in");
                Ok(())
            }
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
            if let Some(_user) = &config.user {
                // TODO:...
                Ok(())
            } else {
                println!("Please login first using 'pet login'");
                Ok(())
            }
        }
        Commands::RemovePet {} => {
            if let Some(_user) = &config.user {
                // TODO:...
                Ok(())
            } else {
                println!("Please login first using 'pet login'");
                Ok(())
            }
        }
        Commands::Status {} => {
            if let Some(_user) = &config.user {
                // TODO:...
                Ok(())
            } else {
                println!("Please login first using 'pet login'");
                Ok(())
            }
        }
        Commands::Feed {} => {
            if let Some(_user) = &config.user {
                // TODO:...
                Ok(())
            } else {
                println!("Please login first using 'pet login'");
                Ok(())
            }
        }
        Commands::Play {} => {
            if let Some(_user) = &config.user {
                // TODO:...
                Ok(())
            } else {
                println!("Please login first using 'pet login'");
                Ok(())
            }
        }
        Commands::AddRepo { path: _path } => {
            if let Some(_user) = &config.user {
                // TODO:...
                Ok(())
            } else {
                println!("Please login first using 'pet login'");
                Ok(())
            }
        }
        Commands::RemoveRepo { path: _path } => {
            if let Some(_user) = &config.user {
                // TODO:...
                Ok(())
            } else {
                println!("Please login first using 'pet login'");
                Ok(())
            }
        }
        Commands::ListRepos {} => {
            if let Some(_user) = &config.user {
                // TODO:...
                Ok(())
            } else {
                println!("Please login first using 'pet login'");
                Ok(())
            }
        }
    };

    // Handle any errors from config operations
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
