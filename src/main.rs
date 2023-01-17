use clap::{Parser, command, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand, Debug)]
enum Commands {
    Normalize {
        #[arg(long, short)]
        path: String
    }
}

fn main() {
    println!("Hello, world!");
}
