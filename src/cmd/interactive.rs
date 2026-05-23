use std::collections::HashSet;

use anyhow::{Context, Result};

use crate::{
    branch::{prep_available_branches, prep_available_remote_branches, prep_available_worktrees},
    config::{Config, RefSource},
    git::read_cached_remote_branches,
    model::Model,
    tui::{AppExitStatus, InteractiveApp, terminal::Terminal},
    types::Worktree,
};

pub fn jump(state: &Model, app_config: Config, active: &Worktree) -> Result<AppExitStatus> {
    let res = {
        let mut t = Terminal::new()?;

        let branches = prep_available_branches(&state.branches, &state.worktrees);

        let local_branches: HashSet<String> = branches.iter().map(|b| b.name.clone()).collect();

        let cached_remote_branches =
            read_cached_remote_branches().context("Could not read local remote cache")?;

        let remotes: Vec<String> = app_config
            .general
            .sources
            .iter()
            .filter_map(|s| match s {
                RefSource::Remote(r) => Some(r.clone()),
                _ => None,
            })
            .collect();

        let remote_branches = prep_available_remote_branches(
            &cached_remote_branches,
            &remotes,
            &local_branches,
            &active.head,
        );

        let worktrees = prep_available_worktrees(&state.worktrees, &state.active_worktree);

        let app = InteractiveApp::new(
            active.head.to_owned(),
            branches,
            remote_branches,
            worktrees,
            app_config,
        );
        app.run(&mut t.terminal)?
    };

    Ok(res)
}
