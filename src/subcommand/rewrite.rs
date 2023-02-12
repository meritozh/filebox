// Copyright (c) 2023 meritozh
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{path::Path, fs::File, io::Read};

use crate::utils::get_canonicalize_path;

use pest::Parser;
use pest_derive::Parser;

struct Rewrite {

}

#[derive(Parser)]
#[grammar = "workflow.pest"]
struct Workflow<'a> {
    nodes: Vec<Node<'a>>
}

impl<'a> Workflow<'a> {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        todo!()
        // let pathbuf = get_canonicalize_path(path.as_ref());
        // let file = File::open(pathbuf).expect("file not exist or denied open");

        // Self { nodes: vec![] }
    }

    fn parse_from(file: File) {
        let pairs = Workflow::parse(Rule::flow, "test here");
        match pairs {
            Ok(paris) => {
                paris.for_each(|p| {
                    match p.as_rule() {
                        Rule::node => todo!(),
                        Rule::normalize => todo!(),
                        Rule::form => todo!(),
                        Rule::recode => todo!(),
                        Rule::target => todo!(),
                        Rule::encoding => todo!(),
                        Rule::rename => todo!(),
                        Rule::command => todo!(),
                        Rule::pattern => todo!(),
                        Rule::inner => todo!(),
                        Rule::flow => todo!(),
                        _ => unreachable!()
                    }
                });
            }
            Err(err) => {
                eprintln!("Failed parsing input: {:}", err);
            }
        }
    }
}

enum Node<'a> {
    Normalize(NormalizeNode<'a>),
    Recode(RecodeNode<'a>),
    Rename(RenameNode<'a>)
}

struct NormalizeNode<'a> {
    from: &'a str,
    to: &'a str
}

struct RecodeNode<'a> {
    target: &'a str,
    encoding: (&'a str, &'a str)
}

enum Pattern<'a> {
    Regex(&'a str),
    Str(&'a str)
}

struct RenameNode<'a> {
    command: &'a str,
    pattern: Pattern<'a>
}
