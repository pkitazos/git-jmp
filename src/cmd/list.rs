use clap::Parser;

use crate::{
    cmd::Run,
    error::GitJumpError,
    git::{fetch_remote_branches, fetch_remotes},
    list::get_active_worktree,
    model::Model,
    print::render_branch_list,
};

#[derive(Debug, Parser)]
/// List all branches
#[clap(visible_alias("ls"))]
pub struct List {
    #[arg(short('r'), long)]
    include_remotes: bool,
}

impl Run for List {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        let active = get_active_worktree(&state.worktrees, &state.active_worktree);
        list_sub_command(state, self.include_remotes).map(|branches| {
            render_branch_list(&active.head, &branches, &state.worktrees);
        })
    }
}

fn list_sub_command(state: &Model, include_remotes: bool) -> Result<Vec<String>, GitJumpError> {
    let mut branches: Vec<String> = state.branches.iter().map(|b| b.name.to_owned()).collect();
    if include_remotes {
        for remote in fetch_remotes()? {
            let mut remote_branches = fetch_remote_branches(&remote)?;
            branches.append(&mut remote_branches);
        }
    }

    Ok(branches)
}
