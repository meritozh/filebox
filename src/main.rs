// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use clap::{command, Parser, Subcommand};
use filebox::subcommand::{normalize, run, workflow};

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

    #[command(about = "Run commands from [<command>.filebox.commands] file")]
    Run {
        #[arg(short, long)]
        path: String,
    },

    #[command(about = "Parse workflow, run every node defined tasks")]
    Workflow {
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
        Command::Run { path } => run::run(path),
        Command::Workflow { path } => {
            let mut workflow = workflow::Workflow::new(path);
            workflow.parse_nodes();
        }
    }
}
