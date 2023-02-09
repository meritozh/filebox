// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{
    fs::{rename, File},
    io::{BufRead, BufReader},
    path::Path,
};

use crate::utils::get_canonicalize_path;

pub fn run<P: AsRef<Path>>(command_filepath: P) {
    let pathbuf = get_canonicalize_path(command_filepath.as_ref());
    let filename = pathbuf.file_name().unwrap();
    let command = filename
        .to_str()
        .map(|name| name.split('.').next().unwrap())
        .expect("[<command>.filebox.commands] file");

    match command {
        "normalize" => {
            let file = File::open(pathbuf).expect("file don't exist or cannot open");
            let stream = BufReader::new(file);

            stream.lines().for_each(|line| {
                if let Ok(line) = line {
                    let mut iter = line.split("=>");
                    let from = iter.next().unwrap();
                    let to = iter.next().unwrap();

                    rename(from, to).expect("rename failed");
                }
            })
        }
        _ => {}
    }
}
