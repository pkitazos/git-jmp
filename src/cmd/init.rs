use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Debug, PartialEq, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    pub fn integration(&self) -> &'static str {
        match self {
            Shell::Bash => include_str!("../../shell/bash.sh"),
            Shell::Zsh => include_str!("../../shell/zsh.sh"),
            Shell::Fish => include_str!("../../shell/fish.fish"),
        }
    }
}

#[derive(Debug, Parser)]
/// Print a shell integration snippet
///
/// Outputs a shell function that wraps git-jmp with worktree directory switching support.
/// Source the output in your shell config to enable the `jmp` shorthand.
#[command(arg_required_else_help = true)]
pub struct Init {
    pub shell: Shell,
}
