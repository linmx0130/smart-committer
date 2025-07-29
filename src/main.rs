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
use nah_chat::{ChatClient, ChatCompletionParamsBuilder, ChatMessage};
use serde_json::json;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use tokio::runtime::{Builder, Runtime};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  #[arg(long, action = clap::ArgAction::SetTrue, help="Reset config file")]
  config: bool,

  commit_file_path: Option<PathBuf>,
}

fn main() -> Result<(), SmartCommitterError> {
  let args = Args::parse();
  if args.config {
    let path = config::UserConfig::create_user_config_template()?;
    println!("Config file is created at {}", path.to_string_lossy());
    println!("Edit it to have correct configuration before using smart-committer!");
    return Ok(());
  }
  let user_config = config::UserConfig::load_user_config()?;
  println!("{:?}", user_config);

  let repo_root = match git::find_repo_root().unwrap() {
    Some(p) => p,
    None => {
      println!("No git repo found.");
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

  let msg = llm_draft_diff_message(diff_content, &user_config.unwrap().llm).unwrap();
  println!("{}", msg);

  Ok(())
}

fn llm_draft_diff_message(
  diff_content: String,
  llm_config: &config::LLMConfig,
) -> Result<String, SmartCommitterError> {
  let tokio_runtime = Builder::new_current_thread()
    .enable_io()
    .enable_time()
    .build()
    .unwrap();

  let user_message = "Based on this git diff output, draft a commit summary to concisely describe the changes. Requirements:
1. The first line should be a title within 50 characters. 
2. Then write a paragraph to describe the changes, what is added, and what is removed. You may use a list of bullet points. 
3. Do not add any other explanation, do not add any field names.
```
{{DIFF_CONTENT}}
```
".replace("{{DIFF_CONTENT}}", &diff_content);

  let mut params = ChatCompletionParamsBuilder::new();
  params.max_token(32768);
  params.insert("enable_thinking", json!(false));
  let messages = vec![ChatMessage {
    role: "user".to_owned(),
    content: user_message,
    reasoning_content: None,
    tool_call_id: None,
    tool_calls: None,
  }];

  tokio_runtime.block_on(async {
    let mut stdout = std::io::stdout();
    let chat_client = ChatClient::init(llm_config.base_url.clone(), llm_config.auth_token.clone());
    let stream = chat_client
      .chat_completion_stream(&llm_config.model, &messages, &params)
      .await
      .unwrap();
    pin_mut!(stream);
    let mut message = ChatMessage::new();
    let mut counter = 0;
    print!("Drafting diff message");
    let _ = stdout.flush();
    while let Some(delta_result) = stream.next().await {
      counter = counter + 1;
      if counter >= 10 {
        counter -= 10;
      }
      print!("\rDrafting diff message");
      for _ in 0..counter {
        let __ = stdout.write_all(b".");
      }
      let _ = stdout.flush();

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
    let _ = stdout.write_all(b"\n");
    Ok(message.content)
  })
}
