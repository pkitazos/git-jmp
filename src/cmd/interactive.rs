use anyhow::Result;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::stdout;

use crate::{
    app::{AppExitStatus, InteractiveApp, TerminalGuard},
    config::Config,
    list::{prep_available_branches, prep_available_worktrees},
    model::Model,
    types::Worktree,
};

pub fn jump(state: &Model, app_config: Config, active: &Worktree) -> Result<AppExitStatus> {
    let res = {
        let _guard = TerminalGuard::enter()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let branches = prep_available_branches(&state.branches, &state.worktrees);
        let worktrees = prep_available_worktrees(&state.worktrees, &state.active_worktree);

        let app = InteractiveApp::new(active.head.to_owned(), branches, worktrees, app_config);
        app.run(&mut terminal)?
    };

    Ok(res)
}
