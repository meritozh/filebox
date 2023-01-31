use std::path::PathBuf;

use clap::{Parser, command, Subcommand};

#[derive(Parser, Debug)]
#[command(author = "meritozh")]
struct Args {
    #[command(subcommand)]
    command: Command
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Normalize file name, convert NFD to NFC")]
    Normalize {
        #[arg(short, long)]
        path: PathBuf
    }
}

fn main() {
    let args = Args::parse();
    
    match args.command {
      Command::Normalize { path } => {

    } 
}
