use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Login to your BitPet account
    Login {},

    /// Logout from your BitPet account
    Logout {},

    /// Add a pet to your BitPet account
    AddPet {},

    /// Get the mood, health, and other details of your pet
    Status {},

    /// Feed your pet (based on your git commits since last feed)
    Feed {},

    /// Play with your pet (Makes it happy)
    Play {},

    /// Add a git repo (will be used to fetch commits for feeding your pet)
    AddRepo {
        #[arg(short, long)]
        path: String,
    },

    /// Remove a git repo (will not be used to fetch commits for feeding your pet)
    RemoveRepo {
        #[arg(short, long)]
        path: String,
    },

    /// List all the git repos from which commits will be fetched for feeding your pet
    ListRepos {},
}

fn main() {
    let args = Args::parse();

    // match args.command {
    //     Commands::Init { name } => match name {
    //         Some(pet_name) => println!("Initializing BitPet project with name: {}", pet_name),
    //         None => println!("Initializing BitPet project with default settings"),
    //     },
    //     Commands::Greet { name, count } => {
    //         for _ in 0..count {
    //             println!("Hello, {}!", name);
    //         }
    //     }
    // }
}
