use crate::{
    git::{fetch_remote_branches, git_command},
    list::generate_ranked_list,
    storage::{delete_jump_data_branch, rename_jump_data_branch, update_branch_last_switch},
    types::{BranchDeleteResult, GitJumpError, Head, Model, RankedSearchList, get_active_worktree},
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

/// side-effect: execute git switch
pub fn jump_to(state: &Model, target: &str, args: &[&str]) -> Result<String, GitJumpError> {
    let current_worktree = get_active_worktree(&state.worktrees, &state.active_worktree);

    if args.is_empty() {
        let stay = match &current_worktree.head {
            Head::Branch(b) => target.eq(&b.name),
            Head::Detached { .. } => true,
        };

        if stay {
            return Ok(format!("Staying on {}", target));
        }
    }

    let err = match git_command("switch", &args) {
        Ok(msg) => return Ok(msg),
        Err(e) => e,
    };

    match fetch_remote_branches() {
        Ok(remote_branch_names) => {
            let target_exists = remote_branch_names
                .iter()
                .map(|x| x.as_str())
                .collect::<Vec<&str>>()
                .contains(&target);

            if target_exists {
                return Err(GitJumpError::SwitchFailed(err));
            }
        }
        Err(err) => return Err(GitJumpError::Other(err)),
    }

    let RankedSearchList { available, .. } = {
        let head = current_worktree.head.clone();
        let branches = state.branches.clone();
        let worktrees = state.worktrees.clone();

        generate_ranked_list(&head, &branches, &worktrees, target)
    };

    if available.is_empty() {
        return Err(GitJumpError::NoMatch {
            target: target.to_string(),
        });
    }

    return switch_to_list_item(&available[0]);
}

/// side-effect: execute `git switch`
pub fn switch_to_list_item(head: &Head) -> Result<String, GitJumpError> {
    match head {
        Head::Detached { sha } => Ok(format!("Staying on {}", sha)),
        Head::Branch(b) => match git_command("switch", &[&b.name]) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(GitJumpError::SwitchFailed(err)),
        },
    }
}
