use clap::{Parser, Subcommand};

use crate::{
    cmd::{
        delete::Delete, interactive::Interactive, jump::JumpTo, list::List, new::New,
        rename::Rename,
    },
    error::GitJumpError,
    model::Model,
    version::check_pkg_version,
};

pub mod delete;
pub mod interactive;
pub mod jump;
pub mod list;
pub mod new;
pub mod rename;
pub mod switch;

pub const NAME: &str = "git-jmp";

pub trait Run {
    fn run(&self, state: &Model) -> Result<(), GitJumpError>;
}

#[derive(Parser)]
#[command(
    name = NAME,
    version,
    about,
    long_about = "\
A fast, interactive branch switcher for Git with fuzzy search, recency sorting, and worktree support.

Run with no arguments to launch the interactive UI. Branches are sorted by recency, the ones
you switch to most often float to the top. Start typing to fuzzy-filter the list.

Jump directly to a branch without the UI by passing a name or partial match:

  git jmp 481       switches to feat/issue-481-auth-refactor
  git jmp signup    switches to feat/user-signup-flow

An exact match is tried first, then the best fuzzy match.",
    propagate_version = true,
    override_usage = "git jmp [BRANCH] | git jmp <COMMAND> | git jmp",
    after_long_help = "\
Interactive mode keybindings:
  Up/Down     Navigate the list
  j/k         Navigate the list (vim mode)
  Enter       Switch to the selected branch
  Type        Fuzzy-filter branches by name
  Alt+0..9    Quick-select a branch by its position

Configuration:
  Global config: ~/.config/git-jmp/config.toml
  Local config:  .jump/config.toml (at repo root, overrides global)

  See https://github.com/pkitazos/git-jmp#configuration for all options.

Support:
  https://github.com/pkitazos/git-jmp"
)]
pub struct Cli {
    /// Jump to a branch by exact or fuzzy name match
    ///
    /// Checks for an exact match first, then falls back to the best fuzzy match.
    /// You can use just part of the name, e.g. `git jmp 481` to match `feat/issue-481-auth-refactor`.
    pub branch: Option<String>,

    /// Enable vim-style navigation (j/k, Normal/Input mode split)
    #[arg(long)]
    pub vim_mode: bool,

    /// Include any branches which exist on any remote in the interactive list
    #[arg(short('r'), long)]
    pub include_remotes: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Ls(List),
    New(New),
    Rm(Delete),
    Mv(Rename),
}

impl Run for Commands {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        match self {
            Commands::Ls(cmd) => cmd.run(state),
            Commands::New(cmd) => cmd.run(state),
            Commands::Rm(cmd) => cmd.run(state),
            Commands::Mv(cmd) => cmd.run(state),
        }
    }
}

pub enum Invocation {
    Interactive(Interactive),
    JumpTo(JumpTo),
    Sub(Commands),
}

impl Run for Invocation {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        match self {
            Invocation::Interactive(cmd) => cmd.run(&state),
            Invocation::JumpTo(cmd) => cmd.run(&state),
            Invocation::Sub(cmd) => cmd.run(&state),
        }
    }
}

impl Cli {
    pub fn into_invocation(self) -> Invocation {
        match (self.command, self.branch) {
            (Some(c), None) => Invocation::Sub(c),

            (None, Some(branch)) => Invocation::JumpTo(JumpTo { branch }),

            (None, None) => Invocation::Interactive(Interactive {
                vim_mode: self.vim_mode,
                include_remotes: self.include_remotes,
            }),

            (Some(_), Some(_)) => unreachable!("clap grammar prevents this"),
        }
    }

    pub fn run(self) -> Result<(), GitJumpError> {
        let state = Model::init()?;

        let check_for_update = state.config.general.auto_check_updates;

        self.into_invocation().run(&state).inspect(|_| {
            if check_for_update {
                check_pkg_version();
            }
        })
    }
}
