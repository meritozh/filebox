// Copyright (c) 2023 meritozh
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use std::{result::Result, path::PathBuf};
use unicode_normalization::UnicodeNormalization;

pub(crate) enum Error {
    SomeError
}

pub(crate) fn conver_nfd_to_nfc_if_need<P: Into<PathBuf>>(path: P) -> Result<(), Error> {
    let path_buf: PathBuf = path.into();

    if path_buf.is_file() {

    }

    Ok(())
}