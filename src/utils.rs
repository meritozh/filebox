// Copyright (c) 2023 meritozh
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::path::{Path, PathBuf};

pub(crate) fn get_canonicalize_path(path: &Path) -> PathBuf {
    if path.starts_with("~") {
        let home: PathBuf = std::env::var("HOME").unwrap().into();
        let removed = path.strip_prefix("~").unwrap();
        return home.join(removed);
    }

    if !path.is_absolute() {
        return path.canonicalize().unwrap();
    }

    path.to_path_buf()
}
