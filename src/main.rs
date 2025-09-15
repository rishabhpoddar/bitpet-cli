use clap::Parser;

mod commands;
use commands::Commands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Login {} => {
            println!("Logging in...");
        }
        Commands::Logout {} => {
            println!("Logging out...");
        }
        Commands::NewPet {} => {
            println!("Adopting a new pet...");
        }
        Commands::RemovePet {} => {
            println!("Letting go of your pet...");
        }
        Commands::Status {} => {
            println!("Getting status of your pet...");
        }
        Commands::Feed {} => {
            println!("Feeding your pet...");
        }
        Commands::Play {} => {
            println!("Playing with your pet...");
        }
        Commands::AddRepo { path } => {
            println!("Adding a git repo: {}", path);
        }
        Commands::RemoveRepo { path } => {
            println!("Removing a git repo: {}", path);
        }
        Commands::ListRepos {} => {
            println!("Listing all git repos...");
        }
    }
}
