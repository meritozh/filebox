// Copyright (c) 2023 meritozh
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{path::Path, fs::File, io::{BufReader, BufRead}};

use crate::utils::get_canonicalize_path;

use pest_derive::Parser;

struct Rewrite {

}

struct Workflow<'a> {
    nodes: Vec<Node<'a>>
}

impl<'a> Workflow<'a> {
    // pub fn new<P: AsRef<Path>>(path: P) -> Self {
    //     let pathbuf = get_canonicalize_path(path.as_ref());
    //     let file = File::open(pathbuf).expect("file not exist or denied open");

    //     Self { nodes: vec![] }
    // }

    fn parse_from(file: File) {
        let stream = BufReader::new(file);

    }
}

#[derive(Parser)]
#[grammar = "workflow.pest"]
struct Parser;

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
