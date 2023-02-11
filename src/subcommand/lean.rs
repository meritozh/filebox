// Copyright (c) 2023 meritozh
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::path::Path;

use walkdir::WalkDir;

use crate::utils::get_canonicalize_path;

fn lean<P: AsRef<Path>>(path: P) {
    let pathbuf = get_canonicalize_path(path.as_ref());
    let walkdir = WalkDir::new(pathbuf);

    walkdir.into_iter().for_each(|file| {
        if let Ok(pathbuf) = file.map(|f| f.into_path()) {
            
        }
    })
}