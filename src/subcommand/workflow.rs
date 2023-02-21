// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{fs::File, io::Read, path::Path};

use crate::utils::get_canonicalize_path;
use crate::subcommand::executor::Executor;

use encoding_rs::Encoding;
use pest::Parser;
use pest_derive::Parser;

type Task = impl Fn(&[u8]) -> String;

#[derive(Parser)]
#[grammar = "workflow.pest"]
pub struct Workflow<'a> {
    source_content: String,
    nodes: Option<Vec<Node<'a>>>,
    executor: Executor<Task>,
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
            executor: Executor::new()
        }
    }

    pub fn parse_nodes(&'a mut self) {
        let pairs = Workflow::parse(Rule::flow, self.source_content.as_str());
        match pairs {
            Ok(paris) => {
                self.nodes = Some(
                    paris
                        .map(|pair| match pair.as_rule() {
                            Rule::path => Node::Path(pair.as_str()),
                            Rule::target => Node::Target(match pair.as_str() {
                                "filename" => Target::Filename,
                                "content" => Target::Content,
                                _ => unreachable!(),
                            }),
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
                                    assert!(matches!(pair.as_rule(), Rule::encoding));
                                    pair.as_str()
                                });
                                Node::Recode(match iter.next().unwrap() {
                                    "AUTO" => RecodeNode { encoding: None },
                                    from @ _ => RecodeNode {
                                        encoding: Some((from, iter.next().unwrap())),
                                    },
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
            let path = nodes
                .iter()
                .find(|n| matches!(n, Node::Path(_)))
                .expect("must provide PATH node");

            nodes.iter().for_each(|node| match node {
                Node::Path(_) => {
                    // Ignore Path node
                }
                Node::Normalize(node) => match (&node.from, &node.to) {
                    (Form::Nfc, Form::Nfd) => {
                        self.executor.add_task(|input| input);
                    }
                    (Form::Nfd, Form::Nfc) => {}
                    _ => unreachable!(),
                },
                Node::Recode(node) => match node.encoding {
                    Some((from, to)) => {
                        let from = Encoding::for_label(from.as_bytes())
                            .expect("from encoding isn't correct");
                        let to =
                            Encoding::for_label(to.as_bytes()).expect("to encoding isn't correct");
                    }
                    None => todo!(),
                },
                Node::Rename(node) => match node.command {
                    Command::Remove => {
                        node.pattern.iter().for_each(|pat| todo!());
                    }
                },
                _ => unreachable!(),
            });
        }
    }
}

enum Node<'a> {
    Path(&'a str),
    Target(Target),
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
    encoding: Option<(&'a str, &'a str)>,
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
