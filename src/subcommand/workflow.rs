// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::io::{self, BufWriter, Write};
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;
use std::{fs::File, io::Read, path::Path};

use crate::utils::{get_canonicalize_path, is_hidden};

use chardetng::EncodingDetector;
use encoding_rs::{Encoding, WINDOWS_1252};
use pest::Parser;
use pest_derive::Parser;

use unicode_normalization::{is_nfc, is_nfd, is_nfkc, is_nfkd, UnicodeNormalization};
use walkdir::WalkDir;

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
                                Node::Recode(RecodeNode {
                                    encoding: (iter.next().unwrap(), iter.next().unwrap()),
                                })
                            }
                            Rule::unbake => Node::Unbake,
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
                                    patterns: iter
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

    pub fn run(&self) -> io::Result<()> {
        if let Some(ref nodes) = self.nodes {
            let path = if let Node::Path(path) = nodes
                .iter()
                .find(|n| matches!(n, Node::Path(_)))
                .expect("must provide PATH node")
            {
                path
            } else {
                unreachable!()
            };

            let target = if let Node::Target(target) = nodes
                .iter()
                .find(|n| matches!(n, Node::Target(_)))
                .expect("must provide TARGET node")
            {
                target
            } else {
                unreachable!()
            };

            if matches!(target, Target::Content) {
                unimplemented!()
            }

            let pathbuf = get_canonicalize_path(path.as_ref());
            let walkdir = WalkDir::new(pathbuf);
            let output = File::create("workflow-results.olog").unwrap();

            let mut stream = BufWriter::new(output);

            walkdir
                .into_iter()
                .filter_entry(|e| !is_hidden(e))
                .for_each(|file| {
                    if let Ok(pathbuf) = file.map(|f| f.into_path()) {
                        let _x = nodes
                            .iter()
                            .fold::<Option<PathBuf>, _>(None, |input, node| {
                                let pathbuf = if let Some(inner) = input {
                                    inner
                                } else {
                                    pathbuf.clone()
                                };

                                let modified_pathbuf = match node {
                                    Node::Path(_) | Node::Target(_) => None,
                                    Node::Normalize(node) => match (&node.from, &node.to) {
                                        (Form::Nfc, Form::Nfd) => convert_to_nfd(&pathbuf),
                                        (Form::Nfd, Form::Nfc) => convert_to_nfc(&pathbuf),
                                        _ => None,
                                    },
                                    Node::Recode(node) => {
                                        let (from, to) = node.encoding;
                                        let from = Encoding::for_label(from.as_bytes())
                                            .expect("from encoding isn't correct");
                                        let to = Encoding::for_label(to.as_bytes())
                                            .expect("to encoding isn't correct");

                                        change_encoding(from, to, &pathbuf)
                                    }
                                    Node::Unbake => unbaking_mojibake(&pathbuf),
                                    Node::Rename(node) => match node.command {
                                        Command::Remove => {
                                            remove_str_by_patterns(&node.patterns, &pathbuf)
                                        }
                                    },
                                };

                                modified_pathbuf.or(Some(pathbuf))
                            });
                    }
                });

            stream.flush()?;
        }
        Ok(())
    }
}

fn convert_to_nfc(path: &Path) -> Option<PathBuf> {
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if is_nfd(filename) || is_nfkd(filename) {
            let normalized_filename = filename.nfc().collect::<String>();
            return Some(path.with_file_name(normalized_filename));
        }
    }

    None
}

fn convert_to_nfd(path: &Path) -> Option<PathBuf> {
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if is_nfc(filename) || is_nfkc(filename) {
            let normalized_filename = filename.nfc().collect::<String>();
            return Some(path.with_file_name(normalized_filename));
        }
    }

    None
}

fn change_encoding(from: &'static Encoding, to: &'static Encoding, path: &Path) -> Option<PathBuf> {
    if let Some(filename) = path.file_name() {
        let (from_encoded, _, _) = from.encode(filename.to_str().unwrap());
        let (to_decoded, _, _) = to.decode(from_encoded.as_ref());
        return Some(path.with_file_name(to_decoded.to_string()));
    }

    None
}

fn unbaking_mojibake(path: &Path) -> Option<PathBuf> {
    if let Some(filename) = path.file_name() {
        let mut detector = EncodingDetector::new();
        detector.feed(filename.as_bytes(), true);
        let (encoding, is_ranked) = detector.guess_assess(None, false);
        if is_ranked {
            return change_encoding(WINDOWS_1252, encoding, path);
        }
    }

    None
}

fn remove_str_by_patterns(patterns: &[Pattern], path: &Path) -> Option<PathBuf> {
    if let Some(filename) = path.file_name() {
        let mut mutable_filename: String = filename.to_str().unwrap().into();
        patterns.iter().for_each(|pat| match pat {
            Pattern::Regex(_regex) => {
                unimplemented!()
            }
            Pattern::Str(str) => mutable_filename.remove_matches(str),
        });

        return Some(path.with_file_name(mutable_filename));
    }

    None
}

enum Node<'a> {
    Path(&'a str),
    Target(Target),
    Normalize(NormalizeNode),
    Recode(RecodeNode<'a>),
    Unbake,
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
    encoding: (&'a str, &'a str),
}

enum Target {
    Filename,
    Content,
}

struct RenameNode<'a> {
    command: Command,
    patterns: Vec<Pattern<'a>>,
}

enum Pattern<'a> {
    Regex(&'a str),
    Str(&'a str),
}

enum Command {
    Remove,
}
