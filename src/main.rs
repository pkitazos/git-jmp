use anyhow::Result;
use clap::Parser;
use std::process::ExitCode;

pub mod app;
pub mod cmd;
pub mod config;
pub mod fuzzy_match;
pub mod git;
pub mod init;
pub mod list;
pub mod print;
pub mod storage;
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
