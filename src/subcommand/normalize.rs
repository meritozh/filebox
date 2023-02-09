// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use cjk::is_cjkish_codepoint;
use encoding_rs::{GBK, WINDOWS_1252};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};
use unicode_normalization::{is_nfd, is_nfkd, UnicodeNormalization};
use walkdir::*;

use crate::utils::get_canonicalize_path;

fn convert_to_nfc(path: &Path) -> PathBuf {
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if is_nfd(filename) || is_nfkd(filename) {
            let normalized_filename = filename.nfc().collect::<String>();
            return path.with_file_name(normalized_filename);
        }
    }

    path.to_path_buf()
}

fn latin1_to_utf8(path: &Path) -> io::Result<PathBuf> {
    if let Some(filename) = path.file_name() {
        let filename_str = filename.to_str().unwrap();
        if !guess_is_cjk(filename_str) {
            let (latin1, _, _) = WINDOWS_1252.encode(filename.to_str().unwrap());
            let (gbk, _, _) = GBK.decode(latin1.as_ref());
            // filter decode twice if guess wrong result.
            if gbk.contains("&#") {
                return Err(io::Error::new(io::ErrorKind::Other, "UTF-8 all ready"));
            }
            return Ok(path.with_file_name(gbk.to_string()));
        }
    }

    Err(io::Error::new(io::ErrorKind::Other, "UTF-8 all ready"))
}

fn guess_is_cjk(str: &str) -> bool {
    let (cjk_count, total_count) = str.chars().fold((0, 0), |(cjk_count, total_count), c| {
        if is_cjkish_codepoint(c) {
            return (cjk_count + 1, total_count + 1);
        }
        (cjk_count, total_count + 1)
    });

    if cjk_count >= total_count / 5 * 4 {
        return true;
    }
    false
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub fn all_to_nfc_and_utf8<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let pathbuf = get_canonicalize_path(path.as_ref());
    let walkdir = WalkDir::new(pathbuf);
    let output = File::create("normalize.filebox.commands")?;

    let mut stream = BufWriter::new(output);

    walkdir
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .for_each(|file| {
            if let Ok(pathbuf) = file.map(|f| f.into_path()) {
                try_to_nfc_and_utf8(pathbuf.as_path())
                    .ok()
                    .and_then(|record| {
                        let from = record.0.to_str()?;
                        let to = record.1.to_str()?;

                        return stream.write_all(format!("{from}=>{to}\n").as_bytes()).ok();
                    });
            }
        });

    stream.flush()?;

    Ok(())
}

fn try_to_nfc_and_utf8(path: &Path) -> io::Result<(&Path, PathBuf)> {
    let nfc_pathbuf = convert_to_nfc(path);
    latin1_to_utf8(nfc_pathbuf.as_path()).map(|new_pathbuf| (path, new_pathbuf))
}
