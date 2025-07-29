/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::{SmartCommitterError, SmartCommitterErrorKind};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

/**
 * User configuration.
 */
#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
  /** Editor config */
  pub editor: EditorConfig,
  /** LLM config */
  pub llm: LLMConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EditorConfig {
  /** Real editor command */
  pub command: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LLMConfig {
  /** Base URL of the LLM server */
  pub base_url: String,

  /** Model name */
  pub model: String,

  /** Auth token of the LLM server */
  pub auth_token: Option<String>,
}

impl UserConfig {
  pub fn load_user_config() -> Result<Option<UserConfig>, SmartCommitterError> {
    let config_file_path = UserConfig::get_config_file_path()?;

    if !config_file_path.exists() {
      return Ok(None);
    }

    let mut config_file = match File::open(&config_file_path) {
      Ok(f) => f,
      Err(e) => {
        return Err(SmartCommitterError {
          kind: SmartCommitterErrorKind::IOError,
          message: format!(
            "Not able to open the config file: {}",
            config_file_path.to_string_lossy()
          ),
          source: Some(Box::new(e)),
        });
      }
    };
    let mut config_content = String::new();
    match config_file.read_to_string(&mut config_content) {
      Ok(_) => {}
      Err(e) => {
        return Err(SmartCommitterError {
          kind: SmartCommitterErrorKind::IOError,
          message: format!(
            "Failed to read the config file: {}",
            config_file_path.to_string_lossy()
          ),
          source: Some(Box::new(e)),
        });
      }
    };

    match toml::from_str(&config_content) {
      Ok(c) => Ok(c),
      Err(e) => Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::IOError,
        message: format!(
          "Failed to parse the config file: {}",
          config_file_path.to_string_lossy()
        ),
        source: Some(Box::new(e)),
      }),
    }
  }

  /**
   * Create user config file and return its path if the creation succeeds.
   */
  pub fn create_user_config_template() -> Result<PathBuf, SmartCommitterError> {
    let config_file_path = UserConfig::get_config_file_path()?;

    let user_config_template = UserConfig {
      editor: EditorConfig {
        command: "nano".to_owned(),
      },
      llm: LLMConfig {
        base_url: "https://api.openai.com/v1".to_owned(),
        model: "gpt-4.1".to_owned(),
        auth_token: Some("<AUTH TOKEN HERE>".to_owned()),
      },
    };

    let mut file = match std::fs::File::create(&config_file_path) {
      Ok(f) => f,
      Err(e) => {
        return Err(SmartCommitterError {
          kind: SmartCommitterErrorKind::IOError,
          message: format!(
            "Failed to create the config file: {}",
            config_file_path.to_string_lossy()
          ),
          source: Some(Box::new(e)),
        });
      }
    };

    let toml_content = toml::to_string(&user_config_template).unwrap();
    match file.write_all(toml_content.as_bytes()) {
      Ok(_) => Ok(config_file_path),
      Err(e) => {
        return Err(SmartCommitterError {
          kind: SmartCommitterErrorKind::IOError,
          message: format!(
            "Failed to write the config file: {}",
            config_file_path.to_string_lossy()
          ),
          source: Some(Box::new(e)),
        });
      }
    }
  }

  /**
   * Return the config file path. If the directories don't exists, create them.
   */
  fn get_config_file_path() -> Result<std::path::PathBuf, SmartCommitterError> {
    let mut config_file_path = match std::env::home_dir() {
      Some(p) => p,
      None => {
        return Err(SmartCommitterError {
          kind: SmartCommitterErrorKind::IOError,
          message: "Home directory not found".to_owned(),
          source: None,
        });
      }
    };

    config_file_path.push(".config");
    config_file_path.push("smart-committer");
    if !config_file_path.exists() {
      match std::fs::create_dir_all(&config_file_path) {
        Ok(()) => {}
        Err(e) => {
          return Err(SmartCommitterError {
            kind: SmartCommitterErrorKind::IOError,
            message: format!(
              "Failed to create config directory: {}",
              config_file_path.to_string_lossy()
            ),
            source: Some(Box::new(e)),
          });
        }
      }
    }

    config_file_path.push("config.toml");

    Ok(config_file_path)
  }
}
