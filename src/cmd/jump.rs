use std::path::Path;

use crate::{
    cmd::Run,
    error::GitJumpError,
    git::git_command,
    list::{generate_ranked_list, get_active_worktree},
    model::Model,
    storage::update_branch_last_switch,
    types::now,
};

pub struct JumpTo {
    pub branch: String,
}

impl Run for JumpTo {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        jump_to(state, &self.branch).map(|res| {
            println!("{res}");
        })
    }
}

/// Jumps to the best matching branch.
///
/// - Short-circuits if the target is the active branch.
/// - `-` is treated as jump to the previous branch
/// - attempts exact match first
/// - falls back to fuzzy match
///
/// side-effect: executes `git switch`, updates jump data
pub fn jump_to(state: &Model, target: &str) -> Result<String, GitJumpError> {
    let current_worktree = get_active_worktree(&state.worktrees, &state.active_worktree);

    if target.eq(current_worktree.head.label()) {
        return Ok(format!("Staying on {}", target));
    }

    if target == "-" {
        return switch_and_record(&state.main_worktree.data_file(), "-");
    }

    match switch_and_record(&state.main_worktree.data_file(), target) {
        Ok(msg) => return Ok(msg),
        Err(e) => {
            if state.branches.iter().any(|b| b.name == target) {
                return Err(e);
            }
        }
    };

    let list = generate_ranked_list(&state.branches, &state.worktrees, target);

    if list.available.is_empty() {
        return Err(GitJumpError::NoMatch {
            target: target.to_string(),
        });
    }

    switch_and_record(&state.main_worktree.data_file(), &list.available[0].name)
}

/// Switches to `target` via `git switch` and records the jump timestamp.
///
/// On success, resolves the checked-out branch name with `rev-parse`
/// and attempts to update the jump data file.
/// If `rev-parse` fails, the switch still succeeds.
///
/// side-effects: executes `git switch`, updates jump data
pub fn switch_and_record(data_file: &Path, target: &str) -> Result<String, GitJumpError> {
    match git_command("switch", &[target]) {
        Ok(msg) => {
            if let Ok(branch_name) = git_command("rev-parse", &["--abbrev-ref", "HEAD"]) {
                update_branch_last_switch(data_file, &branch_name, now())?;
            }
            Ok(msg)
        }
        Err(err) => Err(GitJumpError::SwitchFailed(err)),
    }
}
