use clap::Parser;

use crate::{
    cmd::Run, error::GitJumpError, git::git_command, model::Model,
    print::render_branch_deletion_res, storage::delete_jump_data_branch,
};

#[derive(Debug, Parser)]
/// Delete one or more branches
///
/// Runs `git branch -d` under the hood and removes deleted branches from your jump data.
/// Use -f for branches that haven't been fully merged (equivalent to `git branch -D`).
#[command(
    arg_required_else_help = true,
    override_usage = "git jmp rm [OPTIONS] <BRANCH_NAMES>..."
)]
pub struct Delete {
    /// Branches to delete
    #[arg(num_args = 1.., required = true)]
    branch_names: Vec<String>,
    /// Force-delete branches not yet fully merged
    #[arg(short, long)]
    force: bool,
}

impl Run for Delete {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        delete_sub_command(
            state,
            &self
                .branch_names
                .iter()
                .map(|b| b.as_str())
                .collect::<Vec<&str>>(),
            self.force,
        )
        .and_then(|res| {
            render_branch_deletion_res(&res);
            if res
                .iter()
                .any(|r| matches!(r, BranchDeleteResult::Failed(..)))
            {
                Err(GitJumpError::SilentExit)
            } else {
                Ok(())
            }
        })
    }
}

pub enum BranchDeleteResult {
    Deleted(String),        // branch name
    Failed(String, String), // branch name, reason
}

/// side-effect: update the JumpData file
fn delete_sub_command(
    state: &Model,
    branch_names: &[&str],
    force: bool,
) -> Result<Vec<BranchDeleteResult>, GitJumpError> {
    let delete_flag = if force { "-D" } else { "-d" };

    let results: Vec<_> = branch_names
        .iter()
        .map(|&b| match git_command("branch", &[delete_flag, b]) {
            Ok(_) => BranchDeleteResult::Deleted(b.to_string()),
            Err(e) => BranchDeleteResult::Failed(b.to_string(), e.to_string()),
        })
        .collect();

    let successful_deletions: Vec<_> = results
        .iter()
        .filter_map(|b| match b {
            BranchDeleteResult::Deleted(b) => Some(b.as_str()),
            BranchDeleteResult::Failed(_, _) => None,
        })
        .collect();

    delete_jump_data_branch(&state.main_worktree.data_file(), &successful_deletions)?;
    Ok(results)
}
