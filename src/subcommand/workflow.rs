// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{fs::File, io::Read, path::Path};

use crate::utils::get_canonicalize_path;

use pest::Parser;
use pest_derive::Parser;

struct Rewrite {}

#[derive(Parser)]
#[grammar = "workflow.pest"]
pub struct Workflow<'a> {
    source_content: String,
    nodes: Option<Vec<Node<'a>>>,
}

impl<'a> Workflow<'a> {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let pathbuf = get_canonicalize_path(path.as_ref());

        let mut file = File::open(pathbuf).expect("file not exist or denied open");
        let mut buf = String::new();

        file.read_to_string(&mut buf)
            .expect("must provide UTF-8 encoding workflow file");

        Self {
            source_content: buf,
            nodes: None,
        }
    }

    pub fn parse_nodes(&'a mut self) {
        let pairs = Workflow::parse(Rule::flow, self.source_content.as_str());
        match pairs {
            Ok(paris) => {
                self.nodes = Some(
                    paris
                        .map(|pair| match pair.as_rule() {
                            Rule::normalize => {
                                let mut iter = pair.into_inner().map(|pair| {
                                    assert!(matches!(pair.as_rule(), Rule::form));
                                    match pair.as_str() {
                                        "NFD" => Form::Nfd,
                                        "NFC" => Form::Nfc,
                                        _ => unreachable!(),
                                    }
                                });
                                Node::Normalize(NormalizeNode {
                                    from: iter.next().unwrap(),
                                    to: iter.next().unwrap(),
                                })
                            }
                            Rule::recode => {
                                let mut iter = pair.into_inner().map(|pair| {
                                    assert!(matches!(
                                        pair.as_rule(),
                                        Rule::target | Rule::encoding
                                    ));
                                    pair.as_str()
                                });
                                Node::Recode(RecodeNode {
                                    target: iter
                                        .next()
                                        .map(|target| match target {
                                            "filename" => Target::Filename,
                                            "content" => Target::Content,
                                            _ => unreachable!(),
                                        })
                                        .unwrap(),
                                    encoding: (iter.next().unwrap(), iter.next().unwrap()),
                                })
                            }
                            Rule::rename => {
                                let mut iter = pair.into_inner();

                                let command = iter
                                    .next()
                                    .map(|command| match command.as_str() {
                                        "remove" => Command::Remove,
                                        _ => unreachable!(),
                                    })
                                    .unwrap();

                                Node::Rename(RenameNode {
                                    command,
                                    pattern: iter
                                        .map(|pair| match pair.as_rule() {
                                            Rule::regex => Pattern::Regex(pair.as_str()),
                                            Rule::str => Pattern::Str(pair.as_str()),
                                            _ => unreachable!(),
                                        })
                                        .collect(),
                                })
                            }
                            _ => unreachable!(),
                        })
                        .collect(),
                );
            }
            Err(err) => {
                eprintln!("Failed parsing, error is: {:}", err);
                self.nodes = None;
            }
        }
    }

    pub fn run(&self) {
        if let Some(ref nodes) = self.nodes {
            nodes.iter().for_each(|node| match node {
                Node::Normalize(node) => {
                    match (node.from, node.to) {
                        (Form::Nfc, Form::Nfd) => {
                            
                        },
                        (Form::Nfd, Form::Nfc) => {

                        },
                        _ => unreachable!()
                    }
                },
                Node::Recode(node) => {
                    match node.target {
                        Target::Filename => todo!(),
                        Target::Content => todo!(),
                    }
                },
                Node::Rename(node) => {
                    match node.command {
                        Command::Remove => {
                            node.pattern.iter().for_each(|pat| {
                                todo!()
                            });
                        },
                    }
                },
            });
        }
    }
}

enum Node<'a> {
    Normalize(NormalizeNode),
    Recode(RecodeNode<'a>),
    Rename(RenameNode<'a>),
}

struct NormalizeNode {
    from: Form,
    to: Form,
}

enum Form {
    Nfd,
    Nfc,
}

struct RecodeNode<'a> {
    target: Target,
    encoding: (&'a str, &'a str),
}

enum Target {
    Filename,
    Content,
}

struct RenameNode<'a> {
    command: Command,
    pattern: Vec<Pattern<'a>>,
}

enum Pattern<'a> {
    Regex(&'a str),
    Str(&'a str),
}

enum Command {
    Remove,
}
