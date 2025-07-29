/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::{SmartCommitterError, SmartCommitterErrorKind};
use std::env;
use std::path::PathBuf;

pub fn find_repo_root() -> Result<Option<PathBuf>, SmartCommitterError> {
  let mut path = match env::current_dir() {
    Ok(p) => p,
    Err(e) => {
      return Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::IOError,
        message: "Failed to access the current directory.".to_owned(),
        source: Some(Box::new(e)),
      });
    }
  };
  path.push(".git");
  while !path.exists() {
    path.pop();
    if !path.pop() {
      return Ok(None);
    }
    path.push(".git");
  }
  path.pop();
  Ok(Some(path))
}
