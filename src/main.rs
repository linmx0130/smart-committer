/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod config;
mod error;
use clap::Parser;
use error::SmartCommitterError;
use std::path::PathBuf;

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
  let user_config = config::UserConfig::load_user_config();
  println!("{:?}", user_config);
  Ok(())
}
