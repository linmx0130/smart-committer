/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::{SmartCommitterError, SmartCommitterErrorKind};
use std::env;
use std::path::PathBuf;
use std::process::Command;

/**
 * Find the root of the Git repo of current directory.
 *
 * Return value:
 * * `Ok(path)` -> `path` is the root of the Git repo.
 * * `Ok(None)` -> Current directory is not in a Git repo.
 * * `Err(e)` -> Error happens.
 */
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

/**
 * Get the diff output of the current repo between the staged files and
 * the HEAD.
 */
pub fn get_diff() -> Result<String, SmartCommitterError> {
  let mut command = Command::new("git");
  command.arg("diff").arg("--cached");
  let git_diff_output = match command.output() {
    Ok(output) => output,
    Err(e) => {
      return Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::VCSError,
        message: "Failed to fetch diff output from git.".to_owned(),
        source: Some(Box::new(e)),
      });
    }
  };
  let exit_code = git_diff_output.status.code().unwrap_or(255);
  if exit_code != 0 {
    return Err(SmartCommitterError {
      kind: SmartCommitterErrorKind::VCSError,
      message: format!("VCS failed with exit code: {}", exit_code),
      source: None,
    });
  }

  match String::from_utf8(git_diff_output.stdout) {
    Ok(s) => Ok(s),
    Err(e) => Err(SmartCommitterError {
      kind: SmartCommitterErrorKind::VCSError,
      message: format!("Failed to parse Git diff output."),
      source: Some(Box::new(e)),
    }),
  }
}
