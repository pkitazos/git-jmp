use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

pub mod branch;
pub mod cmd;
pub mod config;
pub mod error;
pub mod fuzzy_match;
pub mod git;
pub mod model;
pub mod print;
pub mod storage;
pub mod tui;
pub mod types;

use crate::{cmd::Cli, print::render_git_jump_error};

pub fn main() -> Result<ExitCode> {
    match Cli::parse().run() {
        Ok(()) => Ok(ExitCode::SUCCESS),
        Err(err) => {
            render_git_jump_error(err);
            Ok(ExitCode::FAILURE)
        }
    }
}
