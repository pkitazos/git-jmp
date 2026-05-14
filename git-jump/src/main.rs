use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use std::{
    io::stdout,
    process::{self, ExitCode},
};

use git_jump::{
    app::{self, InteractiveApp},
    command::{
        delete_sub_command, jump_to, list_sub_command, new_sub_command, rename_sub_command,
        switch_and_record,
    },
    config::{self, PartialConfig},
    list::{prep_available_branches, prep_available_worktrees},
    system::init,
    types::{BranchDeleteResult, Model, get_active_worktree},
    ui::{render_branch_deletion_res, render_branch_list, render_git_jump_error},
};

const NAME: &str = "git-jmp";

#[derive(Parser)]
#[command(
    name = NAME,
    version,
    about,
    propagate_version = true,
    override_usage = "git jmp [BRANCH] | git jmp <COMMAND> | git jmp"
)]
pub struct Cli {
    /// Switches to the branch which fuzzy-matches the string
    ///
    /// When a single argument is provided, `<branch name>` can be just part of the name
    /// - `git jmp` will look for the best matching local branch
    /// if `git switch` doesn't find an exact match.
    pub branch: Option<String>,

    /// Include remote branches (applies to interactive mode, direct jump, and list)
    #[arg(short('r'), long)]
    pub include_remotes: bool,

    /// Include remote branches (applies to interactive mode only)
    #[arg(long)]
    pub vim_mode: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List all branches
    #[clap(visible_alias("ls"))]
    List {
        #[arg(short('r'), long)]
        include_remotes: bool,
    },

    #[command(arg_required_else_help = true)]
    /// Create a new branch called <branch_name>
    New { branch_name: String },

    #[command(visible_alias("rm"), arg_required_else_help = true)]
    /// Delete listed branches
    Delete {
        #[arg(num_args = 1..)]
        branch_names: Vec<String>,
        #[arg(short, long)]
        force: bool,
    },

    #[command(
        visible_alias("mv"),
        arg_required_else_help = true,
        override_usage = "git-jump rename [CURRENT_NAME] <NEW_NAME>"
    )]
    /// Rename branch called <curr_name> to <new_name>
    Rename {
        #[arg(num_args = 1..=2)]
        names: Vec<String>,
    },
}

enum Invocation {
    Interactive,
    JumpTo(String),
    Sub(Commands),
}

impl Cli {
    fn into_invocation(self) -> Invocation {
        match (self.command, self.branch) {
            (Some(c), None) => Invocation::Sub(c),
            (None, Some(b)) => Invocation::JumpTo(b),
            (None, None) => Invocation::Interactive,
            (Some(_), Some(_)) => unreachable!("clap grammar prevents this"),
        }
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(crossterm::event::DisableMouseCapture)?;
        enable_raw_mode()?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}

pub fn main() -> Result<ExitCode> {
    let cli = Cli::parse();

    let global_config = match dirs::config_dir() {
        Some(parent_dir) => config::parse_config(parent_dir.join(NAME).join("config.toml"))?,
        None => PartialConfig::default(),
    };

    let data = init()?;
    let state = Model::new(data)?;

    let local_config = config::parse_config(state.main_worktree.jump_dir().join("config.toml"))?;

    let _app_config = config::merge(global_config, local_config);

    match cli.into_invocation() {
        Invocation::Interactive => {
            let active = get_active_worktree(&state.worktrees, &state.active_worktree);

            let res = {
                let _guard = TerminalGuard::enter()?;
                let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

                let branches = prep_available_branches(&state.branches, &state.worktrees);
                let worktrees = prep_available_worktrees(&state.worktrees, &state.active_worktree);

                let app = InteractiveApp::new(active.head.to_owned(), branches, worktrees);
                app.run(&mut terminal)?
            };

            match res {
                app::AppExitStatus::StayedOnDetached => {
                    println!("Staying on {}", active.head.label())
                }
                app::AppExitStatus::Selected(b) => {
                    match switch_and_record(&state.main_worktree.data_file(), &b.name) {
                        Ok(msg) => {
                            if b.is_head(&active.head) {
                                println!("Staying on {}", active.head.label())
                            } else {
                                println!("{}", msg)
                            }
                        }
                        Err(err) => {
                            render_git_jump_error(err);
                            return Ok(ExitCode::FAILURE);
                        }
                    }
                }
                app::AppExitStatus::LocatedAt(worktree) => {
                    let dir = worktree.dir.to_string_lossy();
                    println!(
                        "{} is checked out at {}\nTo switch: cd {}",
                        worktree.head.label(),
                        dir,
                        dir,
                    );
                }
                app::AppExitStatus::Cancelled => {}
            }
        }

        Invocation::JumpTo(branch) => match jump_to(&state, &branch) {
            Ok(info) => {
                println!("{}", info);
            }
            Err(err) => {
                render_git_jump_error(err);
                return Ok(ExitCode::FAILURE);
            }
        },

        Invocation::Sub(cmd) => match cmd {
            Commands::List { include_remotes } => {
                let active = get_active_worktree(&state.worktrees, &state.active_worktree);
                match list_sub_command(&state, include_remotes) {
                    Ok(branches) => render_branch_list(&active.head, &branches, &state.worktrees),
                    Err(err) => {
                        render_git_jump_error(err);
                        return Ok(ExitCode::FAILURE);
                    }
                }
            }

            Commands::New { branch_name } => {
                match new_sub_command(&state, &branch_name) {
                    Ok(info) => {
                        println!("{}", info);
                    }
                    Err(err) => {
                        render_git_jump_error(err);
                        return Ok(ExitCode::FAILURE);
                    }
                };
            }

            Commands::Delete {
                branch_names,
                force,
            } => {
                match delete_sub_command(
                    &state,
                    &branch_names
                        .iter()
                        .map(|b| b.as_str())
                        .collect::<Vec<&str>>(),
                    force,
                ) {
                    Ok(res) => {
                        render_branch_deletion_res(&res);
                        let exit_code = res
                            .iter()
                            .any(|r| matches!(r, BranchDeleteResult::Failed(..)))
                            as i32;
                        process::exit(exit_code)
                    }
                    Err(err) => {
                        render_git_jump_error(err);
                        return Ok(ExitCode::FAILURE);
                    }
                }
            }

            Commands::Rename { names } => {
                let (current_name, new_name) = match names.as_slice() {
                    [new_name] => (None, new_name),
                    [current_name, new_name] => (Some(current_name.as_str()), new_name),
                    _ => unreachable!("clap grammar prevents this"),
                };

                match rename_sub_command(&state, current_name, new_name) {
                    Ok(info) => {
                        println!("{}", info);
                    }
                    Err(err) => {
                        render_git_jump_error(err);
                        return Ok(ExitCode::FAILURE);
                    }
                }
            }
        },
    }

    Ok(ExitCode::SUCCESS)
}

// ---
