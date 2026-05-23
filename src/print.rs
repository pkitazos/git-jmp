use std::io::{self, IsTerminal};

use crossterm::style::Stylize;

use crate::{
    branch::worktree_branch_names,
    cmd::delete::BranchDeleteResult,
    error::GitJumpError,
    git::RemoteBranch,
    types::{Branch, Head, Worktree},
};

pub const INDEX_PADD: &str = "   ";

pub fn render_branch_list(
    active: &Head,
    branches: &[String],
    remote_branches: &[RemoteBranch],
    worktrees: &[Worktree],
) {
    let mut ws = worktree_branch_names(worktrees);
    ws.remove(&active.label());

    if !io::stdout().is_terminal() {
        for name in branches {
            println!("{name}");
        }
        for b in remote_branches {
            println!("{}/{}", b.remote, b.name);
        }
        return;
    }

    for name in branches {
        if name == active.label() {
            println!(" * {}", active.label().green());
        } else if ws.contains(name.as_str()) {
            println!(" + {}", name.as_str().cyan());
        } else {
            println!("{INDEX_PADD}{name}");
        }
    }
    for b in remote_branches {
        println!(
            "{INDEX_PADD}{}{}",
            format!("{}/", b.remote).as_str().grey(),
            b.name
        );
    }
}

pub fn render_git_jump_error(err: GitJumpError) {
    let (title, body) = match &err {
        GitJumpError::BranchCreation(e) => ("Failed to create branch".to_string(), e.to_string()),

        GitJumpError::BranchRenaming(e) => ("Failed to rename branch".to_string(), e.to_string()),

        GitJumpError::SwitchFailed(e) => ("Failed to switch branch".to_string(), e.to_string()),

        GitJumpError::NoMatch { target } => (
            "No matching branch".to_string(),
            format!("'{}' does not match any branch", target),
        ),

        GitJumpError::DetachedHead => (
            "Detached HEAD".to_string(),
            "specify the branch explicitly".to_string(),
        ),

        GitJumpError::Other(e) => ("Error".to_string(), e.to_string()),

        GitJumpError::SilentExit => return,
    };

    eprintln!("{}", title.red().bold());
    eprintln!("{}", body);
}

pub fn render_branch_deletion_res(res: &[BranchDeleteResult]) {
    let (deleted, failed): (Vec<&BranchDeleteResult>, Vec<&BranchDeleteResult>) = res
        .iter()
        .partition(|r| matches!(r, BranchDeleteResult::Deleted(_)));

    let deleted_names: Vec<&str> = deleted
        .iter()
        .filter_map(|r| match r {
            BranchDeleteResult::Deleted(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();

    let failures: Vec<(&str, &str)> = failed
        .iter()
        .filter_map(|r| match r {
            BranchDeleteResult::Failed(name, reason) => Some((name.as_str(), reason.as_str())),
            _ => None,
        })
        .collect();

    match (deleted_names.is_empty(), failures.is_empty()) {
        // full success
        (false, true) => {
            println!("Deleted {}", deleted_names.join(", "));
        }
        // full failure
        (true, false) => {
            eprintln!("{}", "Failed to delete branches".red().bold());
            for (name, reason) in &failures {
                eprintln!(
                    "   [{}] {}",
                    name.grey().bold(),
                    reason.trim().replace('\n', "\n\t")
                );
            }
        }
        // partial
        (false, false) => {
            println!("Deleted {}", deleted_names.join(", "));
            println!();
            eprintln!("{}", "Failed to delete:".red().bold());
            for (name, reason) in &failures {
                eprintln!(
                    "   [{}] {}",
                    name.grey().bold(),
                    reason.trim().replace('\n', "\n\t")
                );
            }
        }
        // empty input
        (true, true) => unreachable!("clap should prevent this"),
    }
}

pub fn render_successful_switch(branch: &Branch, active_head: &Head, msg: &str) {
    if branch.is_head(&active_head) {
        println!("Staying on {}", active_head.label())
    } else {
        println!("{msg}")
    }
}
