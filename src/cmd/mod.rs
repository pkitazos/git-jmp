use clap::{Parser, Subcommand};
use crossterm::style::Stylize;

use crate::{
    branch::get_active_worktree,
    cmd::{
        delete::Delete,
        interactive::jump,
        jump::{JumpTo, switch_and_record},
        list::List,
        new::New,
        rename::Rename,
    },
    config,
    error::GitJumpError,
    model::Model,
    tui::AppExitStatus,
    version::check_pkg_version,
};

pub mod delete;
pub mod interactive;
pub mod jump;
pub mod list;
pub mod new;
pub mod rename;

pub const NAME: &str = "git-jmp";

pub trait Run {
    fn run(&self, state: &Model) -> Result<(), GitJumpError>;
}

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
    ///   if `git switch` doesn't find an exact match.
    pub branch: Option<String>,

    /// Include remote branches (applies to interactive mode only)
    #[arg(long)]
    pub vim_mode: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    List(List),
    New(New),
    Delete(Delete),
    Rename(Rename),
}

impl Commands {
    fn run(self, state: &Model) -> Result<(), GitJumpError> {
        match self {
            Commands::List(cmd) => cmd.run(state),
            Commands::New(cmd) => cmd.run(state),
            Commands::Delete(cmd) => cmd.run(state),
            Commands::Rename(cmd) => cmd.run(state),
        }
    }
}

pub enum Invocation {
    Interactive,
    JumpTo(JumpTo),
    Sub(Commands),
}

impl Cli {
    pub fn into_invocation(self) -> Invocation {
        match (self.command, self.branch) {
            (Some(c), None) => Invocation::Sub(c),
            (None, Some(branch)) => Invocation::JumpTo(JumpTo { branch }),
            (None, None) => Invocation::Interactive,
            (Some(_), Some(_)) => unreachable!("clap grammar prevents this"),
        }
    }

    pub fn run(self) -> Result<(), GitJumpError> {
        // todo: when I support the `--inlude-remotes` flag, that needs to be passed to `init`
        let state = Model::init()?;

        let mut app_config = config::get(&state.main_worktree.jump_dir())?;
        let check_for_update = app_config.general.auto_check_updates;
        if self.vim_mode {
            app_config.general.vim_mode = true
        }

        let res = match self.into_invocation() {
            Invocation::Interactive => {
                let active = get_active_worktree(&state.worktrees, &state.active_worktree);

                let res = jump(&state, app_config, &active)?;

                match res {
                    AppExitStatus::StayedOnDetached => {
                        println!("Staying on {}", active.head.label());
                        Ok(())
                    }

                    AppExitStatus::Selected(b) => {
                        switch_and_record(&state.main_worktree.data_file(), &b.name).map(|res| {
                            if b.is_head(&active.head) {
                                println!("Staying on {}", active.head.label())
                            } else {
                                println!("{res}")
                            }
                        })
                    }

                    AppExitStatus::LocatedAt(worktree) => {
                        let dir = worktree.dir.to_string_lossy();
                        println!(
                            "{} is checked out at {}\nTo switch: {}",
                            worktree.head.label().cyan(),
                            dir.dark_grey(),
                            format!("cd {dir}").bold(),
                        );
                        Ok(())
                    }

                    AppExitStatus::Cancelled => Ok(()),
                }
            }

            Invocation::JumpTo(cmd) => cmd.run(&state),
            Invocation::Sub(cmd) => cmd.run(&state),
        };

        res.inspect(|_| {
            if check_for_update {
                check_pkg_version();
            }
        })
    }
}
