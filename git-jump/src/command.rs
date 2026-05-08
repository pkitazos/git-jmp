use std::iter;

use anyhow::{Result, anyhow};

use crate::{
    git::{fetch_remote_branches, git_command},
    list::{SearchList, generate_list},
    storage::{delete_jump_data_branch, rename_jump_data_branch, update_branch_last_switch},
    types::{Head, Model, Msg},
    utils::now,
};

// The main thing I've taken away from writing this module is that in the TS version
// I was doing a lot of parsing ad-hoc, just because TS is more forgiving.
// In fact the boundary was still very sloppy I was passing raw-string lists all over the place
//
// Taking a page out of the Cobra book (I'm sure clap does the same thing), but basically
// not going to try and build my own bespoke command dispatcher, which probably means
// that a good amount of these function's signature will change
// Parse, Don't Validate!

// ---

pub fn list_sub_command(state: &Model) -> Result<Msg> {
    let list = {
        let current_worktree = state
            .worktrees
            .iter()
            .find(|&w| w.dir.eq(&state.active_worktree))
            .ok_or(anyhow!("Head should exist"))?;

        let head = current_worktree.head.clone();
        let branches = state.branches.clone();
        let worktrees = state.worktrees.clone();

        generate_list(head, branches, worktrees, "")
    };

    match list {
        SearchList::Idle {
            head,
            available_branches,
            in_worktrees,
        } => {
            let branch_names: Vec<String> = iter::once(head.into_label())
                .chain(available_branches.into_iter().map(|b| b.name))
                .chain(in_worktrees.into_iter().map(|w| w.head.into_label()))
                .collect();

            Ok(Msg::Info(branch_names))
        }
        SearchList::InSearch {
            available,
            worktrees,
        } => {
            let branch_names: Vec<String> = available
                .into_iter()
                .map(|h| h.into_label())
                .chain(worktrees.into_iter().map(|b| b.head.into_label()))
                .collect();

            Ok(Msg::Info(branch_names))
        }
    }
}

/// side-effect: update the JumpData file
pub fn new_sub_command(state: &Model, branch_name: &str) -> Result<Msg> {
    match git_command("switch", &["--create"]) {
        Ok(msg) => {
            update_branch_last_switch(&state.main_worktree, branch_name, now())?;
            Ok(Msg::Info(vec![msg]))
        }
        Err(err) => Ok(Msg::Error {
            title: "Failed to Create new Branch".to_string(),
            body: err.to_string(),
        }),
    }
}

/// side-effect: update the JumpData file
pub fn rename_sub_command(state: &Model, src: &str, target: &str) -> Result<Msg> {
    // validation from old variant moves to call-site
    match git_command("branch", &["--move", src, target]) {
        Ok(msg) => {
            rename_jump_data_branch(&state.main_worktree, src, target)?;
            Ok(Msg::Info(vec![msg]))
        }
        Err(err) => Ok(Msg::Error {
            title: "Failed to Rename Branch".to_string(),
            body: err.to_string(),
        }),
    }
}

/// side-effect: update the JumpData file
pub fn delete_sub_command(state: &Model, branch_names: &[&str]) -> Result<Msg> {
    let args: Vec<&str> = ["--delete"]
        .iter()
        .chain(branch_names)
        .map(|&x| x)
        .collect();

    match git_command("branch", &args) {
        Ok(msg) => {
            delete_jump_data_branch(&state.main_worktree, branch_names)?;
            Ok(Msg::Info(vec![msg]))
        }
        Err(err) => Ok(Msg::Error {
            title: "Failed Branch Deletion".to_string(),
            body: err.to_string(),
        }),
    }
}

/// side-effect: execute git switch
pub fn jump_to(state: &Model, target: &str, args: &[&str]) -> Result<Msg> {
    let current_worktree = state
        .worktrees
        .iter()
        .find(|&w| w.dir.eq(&state.active_worktree))
        .ok_or(anyhow!("Head should exist"))?;

    if args.is_empty() {
        let stay = match &current_worktree.head {
            Head::Branch { name } => target.eq(name),
            Head::Detached { sha: _ } => true,
        };

        if stay {
            return Ok(Msg::Info(vec![format!("Staying on {}", target)]));
        }
    }

    let err = match git_command("switch", &args) {
        Ok(msg) => return Ok(Msg::Info(vec![msg])),
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
                return Ok(Msg::Error {
                    title: "Switch Error".to_string(),
                    body: err.to_string(),
                });
            }
        }
        Err(_) => {
            // seems like the error case for the remote fetch is just swallowed
        }
    }

    let list = {
        let head = current_worktree.head.clone();
        let branches = state.branches.clone();
        let worktrees = state.worktrees.clone();

        generate_list(head, branches, worktrees, target)
    };

    match list {
        SearchList::Idle {
            head: _,
            available_branches: _,
            in_worktrees: _,
        } => return Err(anyhow!("Should not happen?")),

        SearchList::InSearch {
            available,
            worktrees: _,
        } => {
            if available.is_empty() {
                return Ok(Msg::Error {
                    title: "No match".to_string(),
                    body: format!("{} does not match any branch", target),
                });
            }

            return switch_to_list_item(&available[0]);
        }
    }
}

/// side-effect: execute `git switch`
pub fn switch_to_list_item(head: &Head) -> Result<Msg> {
    match head {
        Head::Detached { sha } => Ok(Msg::Info(vec![format!("Staying on {}", sha)])),
        Head::Branch { name } => match git_command("switch", &[name]) {
            Ok(msg) => Ok(Msg::Info(vec![msg])),
            Err(msg) => Ok(Msg::Error {
                title: "Failed to Switch Branch".to_string(),
                body: msg.to_string(),
            }),
        },
    }
}
