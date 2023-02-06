use clap::{command, Parser, Subcommand};
use filebox::subcommand::normalize;

#[derive(Parser, Debug)]
#[command(author = "meritozh")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Normalize file name, convert NFD to NFC, force use UTF-8 encoding")]
    Normalize {
        #[arg(short, long)]
        path: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Normalize { path } => {
            normalize::all_to_nfc_and_utf8(path).unwrap();
        }
    }

    println!("end")
}
