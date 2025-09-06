/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod config;
mod error;
mod git;
use clap::Parser;
use error::{SmartCommitterError, SmartCommitterErrorKind};
use futures_util::{StreamExt, pin_mut};
use mini_poml_rs::MarkdownPomlRenderer;
use nah_chat::{ChatClient, ChatCompletionParamsBuilder, ChatMessage};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::runtime::Builder;

#[derive(Parser, Debug)]
#[command(name = "smart-committer")]
#[command(
  version,
  about = "📰 Draft commit message with LLM",
  long_about = "
Smart-committer scans the code to be committed and generates a nice description 
as the commit message. It may be used as an independent command or an editor
for git.

Github: https://github.com/linmx0130/smart-committer
Copyright 2025 Mengxiao Lin, released under MPL-2.0.
"
)]
struct Args {
  #[arg(long, action = clap::ArgAction::SetTrue, help="Reset config file")]
  config: bool,

  commit_file_path: Option<PathBuf>,
}

fn main() -> Result<(), SmartCommitterError> {
  let args = Args::parse();
  if args.config {
    let path = config::UserConfig::create_user_config_template()?;
    eprintln!("Config file is created at {}", path.to_string_lossy());
    eprintln!("Edit it to have correct configuration before using smart-committer!");
    return Ok(());
  }
  let user_config = match config::UserConfig::load_user_config()? {
    Some(c) => c,
    None => {
      eprintln!(
        "Smart-committer user config is not created! Please use following command to create the user config."
      );
      eprintln!("");
      let program_file_name = std::env::args()
        .next()
        .unwrap_or("smart-committer".to_owned());
      eprintln!("  {} --config", program_file_name);
      eprintln!("");
      std::process::exit(1);
    }
  };
  eprintln!("📰 Smart-committer is using {}", user_config.llm.model);

  let repo_root = match git::find_repo_root()? {
    Some(p) => p,
    None => {
      eprintln!("No git repo found.");
      std::process::exit(1);
    }
  };

  match env::set_current_dir(repo_root) {
    Ok(()) => {}
    Err(e) => {
      return Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::IOError,
        message: "Failed to access the git repo root.".to_owned(),
        source: Some(Box::new(e)),
      });
    }
  }

  let diff_content = git::get_diff()?;
  if diff_content.trim().len() == 0 {
    eprintln!("No change is staged! You may need to run `git add` to add files.");
    std::process::exit(3);
  }

  let msg = llm_draft_diff_message(&diff_content, &user_config)?;

  match &args.commit_file_path {
    Some(f) => editor_mode(&f, &msg, &user_config.editor),
    None => plain_command_mode(&msg),
  }
}

fn plain_command_mode(msg: &str) -> Result<(), SmartCommitterError> {
  let mut temp_file = PathBuf::new();
  temp_file.push(".git");
  temp_file.push("SMART_COMMITTER_MSG");
  save_message_to_file(msg, &temp_file)?;
  git::commit_with_message_file(&temp_file)?;
  Ok(())
}

fn editor_mode(
  commit_file_path: &PathBuf,
  msg: &str,
  editor_config: &config::EditorConfig,
) -> Result<(), SmartCommitterError> {
  let mut commit_file = match std::fs::File::open(commit_file_path) {
    Ok(f) => f,
    Err(e) => {
      return Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::IOError,
        message: format!(
          "Failed to open file: {}",
          commit_file_path.to_string_lossy()
        ),
        source: Some(Box::new(e)),
      });
    }
  };
  let mut commit_file_content = String::new();
  let _ = commit_file.read_to_string(&mut commit_file_content);
  let new_commit_file_content = format!("{}\n{}", msg, commit_file_content);
  save_message_to_file(&new_commit_file_content, commit_file_path)?;

  let mut editor_command_parts = editor_config.command.split(' ');
  let mut editor_command = std::process::Command::new(editor_command_parts.next().unwrap());
  loop {
    match editor_command_parts.next() {
      Some(v) => {
        editor_command.arg(v);
      }
      None => {
        break;
      }
    }
  }
  editor_command.arg(commit_file_path);

  let mut editor = match editor_command.spawn() {
    Ok(child) => child,
    Err(e) => {
      return Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::IOError,
        message: "Failed to launch the editor".to_owned(),
        source: Some(Box::new(e)),
      });
    }
  };
  let status = match editor.wait() {
    Ok(s) => s,
    Err(e) => {
      return Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::IOError,
        message: "Failed to get editor result".to_owned(),
        source: Some(Box::new(e)),
      });
    }
  };

  let exit_code = status.code().unwrap_or(255);
  if exit_code != 0 {
    std::process::exit(exit_code);
  }
  Ok(())
}

fn llm_draft_diff_message(
  diff_content: &str,
  user_config: &config::UserConfig,
) -> Result<String, SmartCommitterError> {
  let tokio_runtime = Builder::new_current_thread()
    .enable_io()
    .enable_time()
    .build()
    .unwrap();

  let mut render_context_map = HashMap::new();
  render_context_map.insert("DIFF_CONTENT".to_string(), json!(diff_content));
  let user_prompt_tempate = config::UserConfig::get_user_prompt_template()?;
  let mut prompt_renderer =
    MarkdownPomlRenderer::create_from_doc_and_variables(&user_prompt_tempate, render_context_map);
  let user_message = prompt_renderer.render().unwrap();

  let mut params = ChatCompletionParamsBuilder::new();
  params.max_tokens(8192);
  user_config.llm.enable_thinking.and_then(|t| {
    params.insert("enable_thinking", json!(t));
    Some(())
  });
  user_config.llm.temperature.and_then(|t| {
    params.temperature(t);
    Some(())
  });
  let messages = vec![ChatMessage {
    role: "user".to_owned(),
    content: user_message,
    reasoning_content: None,
    tool_call_id: None,
    tool_calls: None,
  }];

  tokio_runtime.block_on(async {
    let mut stderr = std::io::stderr();
    let chat_client = ChatClient::init(
      user_config.llm.base_url.clone(),
      user_config.llm.auth_token.clone(),
    );
    let stream = chat_client
      .chat_completion_stream(&user_config.llm.model, &messages, &params)
      .await
      .unwrap();
    pin_mut!(stream);
    let mut message = ChatMessage::new();
    let mut counter = 0;
    eprint!("Drafting diff message");
    let _ = stderr.flush();
    while let Some(delta_result) = stream.next().await {
      counter = counter + 1;
      if counter >= 10 {
        counter -= 10;
      }
      eprint!("\rDrafting diff message");
      for _ in 0..counter {
        let __ = stderr.write_all(b".");
      }
      for _ in counter..10 {
        let __ = stderr.write_all(b" ");
      }
      let _ = stderr.flush();

      match delta_result {
        Ok(delta) => {
          message.apply_model_response_chunk(delta);
        }
        Err(e) => {
          return Err(SmartCommitterError {
            kind: SmartCommitterErrorKind::ModelError,
            message: "Failed to fetch output from the LLM.".to_owned(),
            source: Some(Box::new(e)),
          });
        }
      }
    }
    let _ = stderr.write_all(b"\n");
    Ok(message.content)
  })
}

fn save_message_to_file(content: &str, filename: &PathBuf) -> Result<(), SmartCommitterError> {
  let mut file = match std::fs::File::create(filename) {
    Ok(f) => f,
    Err(e) => {
      return Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::IOError,
        message: format!("Failed to open file: {}", filename.to_string_lossy()),
        source: Some(Box::new(e)),
      });
    }
  };
  match file.write_all(content.as_bytes()) {
    Ok(_) => Ok(()),
    Err(e) => {
      return Err(SmartCommitterError {
        kind: SmartCommitterErrorKind::IOError,
        message: format!("Failed to write file: {}", filename.to_string_lossy()),
        source: Some(Box::new(e)),
      });
    }
  }
}
