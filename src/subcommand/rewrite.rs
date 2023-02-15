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
pub struct Workflow<'a> {
    source_file: String,
    nodes: Option<Vec<Node<'a>>>
}

impl<'a> Workflow<'a> {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let pathbuf = get_canonicalize_path(path.as_ref());
        
        let mut file = File::open(pathbuf).expect("file not exist or denied open");
        let mut buf = String::new();
        
        file.read_to_string(&mut buf);

        Self {
            source_file: buf,
            nodes: None
        }
    }

    pub fn parse_nodes(&self) {
        let pairs = Workflow::parse(Rule::flow, self.source_file.as_str());
        match pairs {
            Ok(paris) => {
                paris.for_each(|p| {
                    println!("{:?}", p.as_str());
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
