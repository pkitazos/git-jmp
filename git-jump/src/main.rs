use anyhow::Result;
use clap::{Parser, Subcommand};

use std::process;

use git_jump::{
    app::InteractiveApp,
    command::{delete_sub_command, jump_to, list_sub_command, new_sub_command, rename_sub_command},
    list::{prep_available_branches, prep_available_worktrees},
    system::init,
    types::{BranchDeleteResult, Model, ModifierKey, get_active_worktree},
    ui::{render_branch_deletion_res, render_branch_list, render_git_jump_error},
};

#[derive(Parser)]
#[command(
    name = "git-jump",
    version,
    about,
    propagate_version = true,
    override_usage = "git jump [BRANCH] | git jump <COMMAND> | git jump"
)]
pub struct Cli {
    /// Switches to the branch which fuzzy-matches the string
    ///
    /// When a single argument is provided, `<branch name>` can be just part of the name
    /// - `git jump` will look for the best matching local branch
    /// if `git switch` doesn't find an exact match.
    pub branch: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List all branches
    #[clap(visible_alias("ls"))]
    List,

    #[command(arg_required_else_help = true)]
    /// Create a new branch called <branch_name>
    New { branch_name: String },

    #[command(visible_alias("rm"), arg_required_else_help = true)]
    /// Delete listed branches
    Delete {
        #[arg(num_args = 1..)]
        branch_names: Vec<String>,
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

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    let data = init()?;

    let (columns, rows) = crossterm::terminal::size()?;

    let modifier_key = if std::env::consts::OS == "macos" {
        ModifierKey::Option
    } else {
        ModifierKey::Alt
    };

    let state: Model = Model {
        main_worktree: data.main_worktree,
        active_worktree: data.active_worktree,
        columns: columns as usize,
        rows: rows as usize,
        max_rows: rows as usize,
        branches: data.branches,
        worktrees: data.worktrees,
        modifier_key: modifier_key,
        interactive_state: None,
    };

    match &cli.into_invocation() {
        Invocation::Interactive => {
            let w = get_active_worktree(&state.worktrees, &state.active_worktree);

            let branches = prep_available_branches(&state.branches, &state.worktrees);
            let worktrees = prep_available_worktrees(&state.worktrees, &state.active_worktree);

            let app = InteractiveApp::new(w.head.to_owned(), branches, worktrees);
            ratatui::run(|terminal| app.run(terminal))?;
        }

        Invocation::JumpTo(branch) => match jump_to(&state, branch, &[]) {
            Ok(info) => {
                println!("{}", info);
                process::exit(0)
            }
            Err(err) => {
                render_git_jump_error(err);
                process::exit(1)
            }
        },

        Invocation::Sub(cmd) => match cmd {
            Commands::List => {
                let active = get_active_worktree(&state.worktrees, &state.active_worktree);
                let branches = list_sub_command(&state);
                render_branch_list(&active.head, &branches, &state.worktrees);
                process::exit(0)
            }

            Commands::New { branch_name } => {
                match new_sub_command(&state, branch_name) {
                    Ok(info) => {
                        println!("{}", info);
                        process::exit(0)
                    }
                    Err(err) => {
                        render_git_jump_error(err);
                        process::exit(1)
                    }
                };
            }

            Commands::Delete { branch_names } => {
                match delete_sub_command(
                    &state,
                    &branch_names
                        .iter()
                        .map(|b| b.as_str())
                        .collect::<Vec<&str>>(),
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
                        process::exit(1)
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
                        process::exit(0)
                    }
                    Err(err) => {
                        render_git_jump_error(err);
                        process::exit(1)
                    }
                }
            }
        },
    }

    Ok(())
}

// ---
