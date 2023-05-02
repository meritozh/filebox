// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::{fs::File, io::Read, path::Path};

use crate::utils::{get_canonicalize_path, is_hidden};

use encoding_rs::Encoding;
use pest::Parser;
use pest_derive::Parser;

use unicode_normalization::{is_nfc, is_nfd, is_nfkc, is_nfkd, UnicodeNormalization};
use walkdir::WalkDir;

#[derive(Parser)]
#[grammar = "workflow.pest"]
pub struct Workflow {
    source_content: String,
}

const GARBLED_CHARS: &str = "&#";
const OVER_DECODED_CHARS: &str = "&#";

impl Workflow {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let pathbuf = get_canonicalize_path(path.as_ref());

        let mut file = File::open(pathbuf).expect("file not exist or denied open");
        let mut buf = String::new();

        file.read_to_string(&mut buf)
            .expect("must provide UTF-8 encoding workflow file");

        Self {
            source_content: buf,
        }
    }

    pub fn get_tokens(&self) -> Option<Vec<Token<'_>>> {
        let pairs = Workflow::parse(Rule::flow, self.source_content.as_str());
        match pairs {
            Ok(paris) => {
                let tokens = paris
                    .map(|pair| {
                        let rule = pair.as_rule();
                        match rule {
                            Rule::path => Token::Path(pair.as_str()),
                            Rule::target => Token::Target(match pair.as_str() {
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
                                Token::Normalize(Normalize {
                                    from: iter.next().unwrap(),
                                    to: iter.next().unwrap(),
                                })
                            }
                            Rule::unbake => {
                                let mut iter = pair.into_inner().map(|pair| {
                                    assert!(matches!(pair.as_rule(), Rule::encoding));
                                    pair.as_str()
                                });
                                Token::Unbake(Unbake {
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

                                Token::Rename(Rename {
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
                        }
                    })
                    .collect();

                Some(tokens)
            }
            Err(err) => {
                eprintln!("Failed parsing, error is: {:}", err);
                None
            }
        }
    }
}

pub fn execute(tokens: Option<Vec<Token<'_>>>) -> io::Result<()> {
    if let Some(ref nodes) = tokens {
        let path = if let Token::Path(path) = nodes
            .iter()
            .find(|n| matches!(n, Token::Path(_)))
            .expect("must provide PATH node")
        {
            path
        } else {
            unreachable!()
        };

        let target = if let Token::Target(target) = nodes
            .iter()
            .find(|t| matches!(t, &&Token::Target(_)))
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
            .for_each(|e| {
                if let Ok(e) = e {
                    if e.file_type().is_dir() {
                        return;
                    }
                    let pathbuf = e.into_path();
                    let final_pathbuf =
                        nodes
                            .iter()
                            .fold::<Option<PathBuf>, _>(None, |input, node| {
                                let pathbuf = if let Some(inner) = input {
                                    inner
                                } else {
                                    pathbuf.clone()
                                };

                                let modified_pathbuf = match node {
                                    Token::Path(_) | Token::Target(_) => None,
                                    Token::Normalize(node) => match (&node.from, &node.to) {
                                        (Form::Nfc, Form::Nfd) => convert_to_nfd(&pathbuf),
                                        (Form::Nfd, Form::Nfc) => convert_to_nfc(&pathbuf),
                                        _ => None,
                                    },
                                    Token::Unbake(node) => {
                                        let (from, to) = node.encoding;
                                        let from = Encoding::for_label(from.as_bytes())
                                            .expect("from encoding isn't correct");
                                        let to = Encoding::for_label(to.as_bytes())
                                            .expect("to encoding isn't correct");

                                        change_encoding(from, to, &pathbuf)
                                    }
                                    Token::Rename(node) => match node.command {
                                        Command::Remove => {
                                            remove_str_by_patterns(&node.patterns, &pathbuf)
                                        }
                                    },
                                };

                                modified_pathbuf.or(Some(pathbuf))
                            });

                    if let Some(final_pathbuf) = final_pathbuf {
                        if final_pathbuf != pathbuf {
                            let from = pathbuf.to_str().unwrap();
                            let to = pathbuf.to_str().unwrap();
                            stream
                                .write_all(format!("{from}=>{to}\n").as_bytes())
                                .unwrap();
                        }
                    }
                }
            });
        stream.flush()?;
    }
    Ok(())
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

        // if decoded twice, just throw result.
        if to_decoded.contains(OVER_DECODED_CHARS) {
            return None;
        }

        return Some(path.with_file_name(to_decoded.to_string()));
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

pub enum Token<'a> {
    Path(&'a str),
    Target(Target),
    Normalize(Normalize),
    Unbake(Unbake<'a>),
    Rename(Rename<'a>),
}

pub struct Normalize {
    from: Form,
    to: Form,
}

enum Form {
    Nfd,
    Nfc,
}

pub struct Unbake<'a> {
    encoding: (&'a str, &'a str),
}

pub enum Target {
    Filename,
    Content,
}

pub struct Rename<'a> {
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
