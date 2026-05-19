use anyhow::Result;

use crate::{
    branch::{prep_available_branches, prep_available_worktrees},
    config::Config,
    model::Model,
    tui::{AppExitStatus, InteractiveApp, terminal::Terminal},
    types::Worktree,
};

pub fn jump(state: &Model, app_config: Config, active: &Worktree) -> Result<AppExitStatus> {
    let res = {
        let mut t = Terminal::new()?;

        let branches = prep_available_branches(&state.branches, &state.worktrees);
        let worktrees = prep_available_worktrees(&state.worktrees, &state.active_worktree);

        let app = InteractiveApp::new(active.head.to_owned(), branches, worktrees, app_config);
        app.run(&mut t.terminal)?
    };

    Ok(res)
}
