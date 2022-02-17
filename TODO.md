# TODO

## 0.1.1

- Fix warning from most recent `clap` version:
  ```
  warning: use of deprecated variant `clap::AppSettings::TrailingVarArg`: Replaced with `Command::trailing_var_arg` and `Command::is_trailing_var_arg_set`
     --> src/config.rs:468:78
      |
  468 |     #[clap(author, version, about, long_about = None, setting = AppSettings::TrailingVarArg)]
      |                                                                              ^^^^^^^^^^^^^^
      |
      = note: `#[warn(deprecated)]` on by default
  ```

## Pre-0.2

- manpage
  - generate from markdown?
- crates.io

## 0.2

- Allow user to provide custom format for title and message
- Update notifications with progress
- Write summary to stdout
  - exit code, time took, etc.
