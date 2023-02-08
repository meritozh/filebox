// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{
    fs::{rename, File},
    io::{BufRead, BufReader},
    path::Path,
};

pub fn run<P: AsRef<Path>>(command_filepath: P) {
    let filename = command_filepath.as_ref().file_name().unwrap();
    let command = filename
        .to_str()
        .map(|name| name.split('.').next().unwrap())
        .expect("[<command>.filebox.commands] file");

    match command {
        "normalize" => {
            let file = File::open(command_filepath).expect("file don't exist or cannot open");
            let stream = BufReader::new(file);

            stream.lines().for_each(|line| {
                if let Ok(line) = line {
                    let mut iter = line.split("=>");
                    let from = iter.next().unwrap();
                    let to = iter.next().unwrap().trim_end();

                    rename(from, to).expect("rename failed");
                }
            })
        }
        _ => {}
    }
}
