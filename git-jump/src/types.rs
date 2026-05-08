use std::{cmp, path::PathBuf};

use crate::{app::InteractiveApp, utils::now};

// AppConfig is not the best name. This is more like.. runtime info?
// - cols, rows, max_rows are definitely just runtime information
// - modifier_key is presentation info, previously it was a `isMac` boolean flag
//   that just changed the string we displayed for the modifier key (opt or alt)
// - main_worktree is actually constant every time
// - active_worktree depends on where you run it from
pub struct Model {
    pub main_worktree: PathBuf,
    pub active_worktree: PathBuf,
    pub modifier_key: ModifierKey,
    pub columns: usize,
    pub rows: usize,
    pub max_rows: usize,
    pub branches: Vec<Branch>,
    pub worktrees: Vec<Worktree>,
    pub interactive_state: Option<InteractiveApp>,
}

pub enum ModifierKey {
    Alt,
    Option,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Branch {
    pub name: String,
    pub last_switch: u64,
}

impl Branch {
    pub fn into_head(self) -> Head {
        Head::Branch {
            name: self.name,
            last_switch: self.last_switch,
        }
    }
}

impl PartialOrd for Branch {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Branch {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        other
            .last_switch
            .cmp(&self.last_switch)
            .then_with(|| self.name.cmp(&other.name))
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Worktree {
    pub dir: PathBuf,
    pub head: Head,
}

impl PartialOrd for Worktree {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Worktree {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.head
            .cmp(&other.head)
            .then_with(|| self.dir.cmp(&other.dir))
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum Head {
    Detached { sha: String },
    Branch { name: String, last_switch: u64 },
}

impl Head {
    pub fn label(&self) -> &str {
        match self {
            Head::Detached { sha } => sha,
            Head::Branch { name, .. } => name,
        }
    }

    pub fn into_label(self) -> String {
        match self {
            Head::Detached { sha } => sha,
            Head::Branch { name, .. } => name,
        }
    }

    pub fn last_switched(&self) -> u64 {
        match self {
            Head::Detached { .. } => now(),
            Head::Branch { last_switch, .. } => last_switch.to_owned(),
        }
    }
}

impl PartialOrd for Head {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Head {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match (self, other) {
            (Head::Detached { sha: a }, Head::Detached { sha: b }) => a.cmp(b),
            (Head::Detached { .. }, Head::Branch { .. }) => cmp::Ordering::Greater,
            (Head::Branch { .. }, Head::Detached { .. }) => cmp::Ordering::Less,
            (
                Head::Branch {
                    name: a_name,
                    last_switch: a_last_switch,
                },
                Head::Branch {
                    name: b_name,
                    last_switch: b_last_switch,
                },
            ) => a_last_switch
                .cmp(b_last_switch)
                .then_with(|| a_name.cmp(b_name)),
        }
    }
}

#[derive(Clone)]
pub struct RankedSearchList {
    /// all branches you can jump to that match the search input
    pub available: Vec<Head>,
    /// all worktrees that match the search input
    pub worktrees: Vec<Worktree>,
}

// the Non-interactive mode really just renders things on-demand
// and may not even need this type at all
pub enum Msg {
    Info(Vec<String>),
    Error { title: String, body: String },
}
