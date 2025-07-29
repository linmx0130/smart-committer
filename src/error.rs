/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SmartCommitterErrorKind {
  IOError,
  VCSError,
  ModelError,
}

#[derive(Debug)]
pub struct SmartCommitterError {
  pub source: Option<Box<dyn Error>>,
  pub kind: SmartCommitterErrorKind,
  pub message: String,
}

impl fmt::Display for SmartCommitterError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match &self.kind {
      SmartCommitterErrorKind::IOError => {
        write!(f, "IO Error: {}", self.message)?;
      }
      SmartCommitterErrorKind::VCSError => {
        write!(f, "VCS Error: {}", self.message)?;
      }
      SmartCommitterErrorKind::ModelError => {
        write!(f, "Model Error: {}", self.message)?;
      }
    }
    match self.source.as_ref() {
      Some(source) => {
        write!(f, "\n")?;
        (source as &dyn fmt::Display).fmt(f)?;
      }
      None => {}
    }
    Ok(())
  }
}

impl Error for SmartCommitterError {}
