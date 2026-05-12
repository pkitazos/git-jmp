use std::path::Path;

use crate::{
    git::git_command,
    list::generate_ranked_list,
    storage::{delete_jump_data_branch, rename_jump_data_branch, update_branch_last_switch},
    types::{BranchDeleteResult, GitJumpError, Head, Model, get_active_worktree},
    utils::now,
};

pub fn list_sub_command(state: &Model) -> Vec<String> {
    state.branches.iter().map(|b| b.name.to_owned()).collect()
}

/// side-effect: update the JumpData file
pub fn new_sub_command(state: &Model, branch_name: &str) -> Result<String, GitJumpError> {
    match git_command("switch", &["--create", branch_name]) {
        Ok(msg) => {
            update_branch_last_switch(&state.main_worktree.data_file(), branch_name, now())?;
            Ok(msg)
        }
        Err(err) => Err(GitJumpError::BranchCreation(err)),
    }
}

/// side-effect: update the JumpData file
pub fn rename_sub_command(
    state: &Model,
    src: Option<&str>,
    target: &str,
) -> Result<String, GitJumpError> {
    let src = match src {
        Some(name) => name.to_owned(),
        None => match get_active_worktree(&state.worktrees, &state.active_worktree).head {
            Head::Branch(b) => b.name.clone(),
            Head::Detached { .. } => return Err(GitJumpError::DetachedHead),
        },
    };

    match git_command("branch", &["--move", &src, target]) {
        Ok(msg) => {
            rename_jump_data_branch(&state.main_worktree.data_file(), &src, target)?;
            Ok(msg)
        }
        Err(err) => Err(GitJumpError::BranchRenaming(err)),
    }
}

/// side-effect: update the JumpData file
pub fn delete_sub_command(
    state: &Model,
    branch_names: &[&str],
) -> Result<Vec<BranchDeleteResult>, GitJumpError> {
    let results: Vec<_> = branch_names
        .iter()
        .map(|&b| match git_command("branch", &["--delete", b]) {
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

    return switch_and_record(&state.main_worktree.data_file(), &list.available[0].name);
}

/// Switches to `target` via `git switch` and records the jump timestamp.
///
/// On success, resolves the checked-out branch name with `rev-parse`
/// and attempts to update the jump data file.
/// If `rev-parse` fails, the switch still succeeds.
///
/// side-effects: executes `git switch`, updates jump data
pub fn switch_and_record(data_file: &Path, target: &str) -> Result<String, GitJumpError> {
    match git_command("switch", &[&target]) {
        Ok(msg) => {
            if let Ok(branch_name) = git_command("rev-parse", &["--abbrev-ref", "HEAD"]) {
                update_branch_last_switch(data_file, &branch_name, now())?;
            }
            Ok(msg)
        }
        Err(err) => Err(GitJumpError::SwitchFailed(err)),
    }
}
