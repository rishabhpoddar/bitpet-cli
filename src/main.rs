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
