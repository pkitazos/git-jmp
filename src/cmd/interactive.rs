use anyhow::{Context, Result};
use crossterm::style::Stylize;
use std::{collections::HashSet, env, fs, process};

use crate::{
    branch::get_active_worktree,
    cmd::{
        Run,
        switch::{switch_and_record, switch_to_remote_and_record},
    },
    config::{Config, RefSource},
    error::GitJumpError,
    git::fetch_remotes,
    model::Model,
    print::render_successful_switch,
    tui::{AppExitStatus, InteractiveApp, terminal::Terminal},
    types::Head,
};

pub struct Interactive {
    pub vim_mode: bool,
    pub include_remotes: bool,
}

impl Run for Interactive {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        let mut app_config = state.config.clone();

        if self.vim_mode {
            app_config.general.vim_mode = true
        }

        if self.include_remotes {
            let refs: HashSet<RefSource> = fetch_remotes()
                .unwrap_or_default()
                .into_iter()
                .map(RefSource::Remote)
                .collect();

            app_config.general.sources = HashSet::from_iter(app_config.general.sources)
                .union(&refs)
                .cloned()
                .collect();
        }

        let active = get_active_worktree(&state.worktrees, &state.active_worktree);

        launch_tui(state, app_config, &active.head)?.apply(state, &active.head)
    }
}

fn launch_tui(state: &Model, app_config: Config, active_head: &Head) -> Result<AppExitStatus> {
    let mut t = Terminal::new()?;
    let app = InteractiveApp::new(state, &app_config, active_head)?;
    app.run(&mut t.terminal)
}

impl AppExitStatus {
    pub fn apply(&self, state: &Model, active_head: &Head) -> Result<(), GitJumpError> {
        match self {
            AppExitStatus::StayedOnDetached => {
                println!("Staying on {}", active_head.label());
                Ok(())
            }

            AppExitStatus::SelectedLocal(b) => {
                switch_and_record(&state.main_worktree.data_file(), &b.name)
                    .map(|msg| render_successful_switch(b, active_head, &msg))
            }

            AppExitStatus::SelectedRemote(b, remote) => {
                switch_to_remote_and_record(&state.main_worktree.data_file(), &b.name, remote)
                    .map(|msg| render_successful_switch(b, active_head, &msg))
            }

            AppExitStatus::LocatedAt(worktree) => {
                let dir = worktree.dir.to_string_lossy();

                match env::var("GIT_JMP_SHELL_INTEGRATION") {
                    Ok(path) if !path.is_empty() => {
                        fs::write(&path, dir.as_ref())
                            .context("Could not change into worktree directory")?;
                        eprintln!("Switched to worktree at {}", dir.bold());
                        process::exit(3);
                    }
                    _ => {
                        println!(
                            "{} is checked out at {}\nTo switch: {}",
                            worktree.head.label().cyan(),
                            dir.dark_grey(),
                            format!("cd {dir}").bold(),
                        );
                        Ok(())
                    }
                }
            }

            AppExitStatus::Cancelled => Ok(()),
        }
    }
}
