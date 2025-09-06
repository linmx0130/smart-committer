smart-committer
===
Draft commit message with LLM

## Introduction
Smart-committer is for people who are still write code nowadays. This simple
tool will send the staged changes to LLM to generate a piece of nice commit
message to describe the changes, and allow further editing before submitting.

Everything is configurable with the TOML config file at 
```
$HOME/.config/smart-committer/config.toml
```

Only `git` is supported now. More VCS supports are coming soon.

## Quick start
You can download a binary from [Release](https://github.com/linmx0130/smart-committer/releases).
Then put the file in a local binary path. For example, on Linux,

```
$ tar xvf smart-committer-linux-x86_64.tar.gz
$ sudo mv scommit /usr/local/bin/scommit
```

Then initialize the configuration with the following commands. You may replace `vi` with the editor you love.
```
$ scommit --config
$ vi $HOME/.config/smart-committer/config.toml
```

If you want to try the latest version on the main branch, clone this repo and build with cargo:
```
$ git clone git@github.com:linmx0130/smart-committer.git
$ cd smart-committer && cargo build --release
$ sudo mv target/release/scommit /usr/local/bin/scommit
```

### Customized prompt
A default prompt that asks the model to generate a summary based on the Git output can be found 
in [src/config.rs](src/config.rs). If you want to use a customized prompt, draft a 
[POML](https://microsoft.github.io/poml/latest/) file as `$HOME/.config/smart-committer/user_prompt.poml`.

The git diff output will be parsed as variable `DIFF_CONTENT` and can be referenced in the POML code. 
Smart-committer uses [`mini-poml-rs`](https://github.com/linmx0130/mini-poml-rs) to parse the POML
code, which may not support all POML features. See the introduction in `mini-poml-rs` for more details.

### Learn more
See [our wiki](https://github.com/linmx0130/smart-committer/wiki) for more documents.

## Copyright
Copyright 2025, Mengxiao Lin. Released with MPL-2.0. Check [LICENSE](LICENSE) file.
