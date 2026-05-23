use clap::Parser;

use crate::{
    cmd::Run, error::GitJumpError, git::git_command, model::Model,
    storage::update_branch_last_switch, types::now,
};

#[derive(Debug, Parser)]
/// Create a new branch and switch to it
///
/// Runs `git switch --create` under the hood and records the new branch
/// as the most recently visited in your jump data.
#[command(
    arg_required_else_help = true,
    override_usage = "git jmp new <BRANCH_NAME>"
)]
pub struct New {
    /// Name for the new branch
    branch_name: String,
}

impl Run for New {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        new_sub_command(state, &self.branch_name).map(|res| {
            println!("{res}");
        })
    }
}

/// side-effect: update the JumpData file
fn new_sub_command(state: &Model, branch_name: &str) -> Result<String, GitJumpError> {
    match git_command("switch", &["--create", branch_name]) {
        Ok(msg) => {
            update_branch_last_switch(&state.main_worktree.data_file(), branch_name, now())?;
            Ok(msg)
        }
        Err(err) => Err(GitJumpError::BranchCreation(err)),
    }
}
