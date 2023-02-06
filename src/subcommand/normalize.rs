// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use cjk::is_simplified_chinese;
use encoding_rs::{GBK, WINDOWS_1252};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use unicode_normalization::{is_nfd, is_nfkd, UnicodeNormalization};

fn get_canonicalize_path(path: &Path) -> io::Result<PathBuf> {
    if path.starts_with("~") {
        let home: PathBuf = std::env::var("HOME").unwrap().into();
        let removed = path.strip_prefix("~").unwrap();
        return Ok(home.join(removed));
    }

    if !path.is_absolute() {
        return Ok(path.canonicalize().unwrap());
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "Canonicalize all ready!",
    ))
}

fn convert_to_nfc(path: &Path) -> io::Result<PathBuf> {
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if is_nfd(filename) || is_nfkd(filename) {
            let normalized_filename = filename.nfc().collect::<String>();
            return Ok(path.with_file_name(normalized_filename));
        }
    }

    Err(io::Error::new(io::ErrorKind::Other, "NFC all ready!"))
}

fn latin1_to_utf8(path: &Path) -> io::Result<PathBuf> {
    if let Some(filename) = path.file_name() {
        if filename != ".DS_Store" && !is_simplified_chinese(filename.to_str().unwrap()) {
            let (latin1, _, _) = WINDOWS_1252.encode(filename.to_str().unwrap());
            let (gbk, _, _) = GBK.decode(latin1.as_ref());
            return Ok(path.with_file_name(gbk.to_string()));
        }
    }

    Err(io::Error::new(io::ErrorKind::Other, "UTF-8 all ready"))
}

pub fn all_to_nfc_and_utf8<P: AsRef<Path> + Into<PathBuf>>(path: P) -> io::Result<()> {
    get_canonicalize_path(path.as_ref())
        .or(Ok(path.into()))
        .map(|path| {
            if path.is_dir() {
                if let Ok(entry) = path.read_dir() {
                    entry
                        .filter_map(|e| {
                            e.ok().and_then(|e| {
                                let path = e.path();
                                if path.is_file() {
                                    return Some(path);
                                }
                                None
                            })
                        })
                        .for_each(|file| {
                            try_to_nfc_and_utf8(file.as_path());
                        });
                }
            } else if path.is_file() {
                try_to_nfc_and_utf8(path.as_ref());
            }
        })
}

fn try_to_nfc_and_utf8(path: &Path) {
    _ = convert_to_nfc(path)
        .or(Ok(path.to_path_buf()))
        .and_then(|pathbuf| latin1_to_utf8(pathbuf.as_path()))
        .and_then(|utf8| fs::rename(path, utf8))
}
