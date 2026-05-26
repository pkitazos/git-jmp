use clap::{Parser, Subcommand};

use crate::{
    cmd::{
        delete::Delete, init::Init, interactive::Interactive, jump::JumpTo, list::List, new::New,
        rename::Rename,
    },
    error::GitJumpError,
    model::Model,
    version::check_pkg_version,
};

pub mod delete;
pub mod init;
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
    about = "A fast, interactive branch switcher for Git.",
    long_about = "\
A fast, interactive branch switcher for Git with fuzzy search, recency sorting, and worktree support.

Run with no arguments to launch the interactive UI. Branches are sorted by recency, the ones
you switch to most often float to the top. Start typing to fuzzy-filter the list.

Jump directly to a branch without the UI by passing a name or partial match:

  git jmp 481       switches to feat/issue-481-auth-refactor
  git jmp signup    switches to feat/user-signup-flow
  git jmp -         switches to the previously checked-out branch

An exact match is tried first, then the best fuzzy match.",
    override_usage = "git jmp [BRANCH]\n       git jmp <COMMAND>",
    after_long_help = "\
Interactive mode keybindings:
  General:
    Enter           Switch to the selected branch
    Ctrl+C          Cancel and exit
    Alt+0..9        Quick-select a branch by its position
                    (⌥+0..9 on macOS)

  Navigation:
    ↑ / ↓           Move up/down the list
    j / k           Move up/down (vim mode only)

  Search input (default mode, or Input mode in vim):
    Type            Fuzzy-filter branches by name
    Alt+← / Alt+→   Move cursor by one word
    Home / Ctrl+A   Jump to start of input
    End  / Ctrl+E   Jump to end of input
    Alt+Backspace   Delete previous word
    Ctrl+U          Delete from cursor to start of input
    Ctrl+K          Delete from cursor to end of input
    Ctrl+W          Clear the entire input

  Vim mode only:
    i               Enter Input mode (to type a search)
    Esc             Return to Normal mode
    q               Cancel and exit (Normal mode only)

Configuration:
  Global config: ~/.config/git-jmp/config.toml
  Local config:  .jump/config.toml (at repo root, overrides global field-by-field)

  See https://github.com/pkitazos/git-jmp#configuration for all options.

Support:
  https://github.com/pkitazos/git-jmp"
)]
pub struct Cli {
    /// Branch to jump to (name, fuzzy match, or `-` for previous)
    ///
    /// Checks for an exact match first, then falls back to the best fuzzy match.
    /// You can use just part of the name, e.g. `git jmp 481` to match
    /// `feat/issue-481-auth-refactor`. Pass `-` to jump back to the previously
    /// checked-out branch.
    pub branch: Option<String>,

    /// Vim navigation
    ///
    /// Enables vim-style navigation (j/k, Normal/Input mode split). Only applies
    /// when launching the interactive UI; ignored if a BRANCH or subcommand is
    /// given.
    #[arg(long, help_heading = "Interactive mode options")]
    pub vim_mode: bool,

    /// Include remote branches
    ///
    /// Includes any branches which exist on any remote in the interactive list.
    /// Only applies when launching the interactive UI; ignored if a BRANCH or
    /// subcommand is given.
    #[arg(short('r'), long, help_heading = "Interactive mode options")]
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
    Init(Init),
}

impl Run for Commands {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        match self {
            Commands::Ls(cmd) => cmd.run(state),
            Commands::New(cmd) => cmd.run(state),
            Commands::Rm(cmd) => cmd.run(state),
            Commands::Mv(cmd) => cmd.run(state),
            Commands::Init(_) => unreachable!("Init is handled before Model::init()"),
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
            Invocation::Interactive(cmd) => cmd.run(state),
            Invocation::JumpTo(cmd) => cmd.run(state),
            Invocation::Sub(cmd) => cmd.run(state),
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
        match self.into_invocation() {
            Invocation::Sub(Commands::Init(Init { shell })) => {
                println!("{}", shell.integration());
                Ok(())
            }
            invocation => {
                let state = Model::init()?;

                invocation.run(&state).inspect(|_| {
                    if state.config.general.auto_check_updates {
                        check_pkg_version();
                    }
                })
            }
        }
    }
}
