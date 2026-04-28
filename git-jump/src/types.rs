use std::{cmp, path::PathBuf};

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
    pub interactive_state: Option<UIState>,
}

pub enum ModifierKey {
    Alt,
    Option,
}

#[derive(PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    pub last_switch: u64,
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

#[derive(PartialEq, Eq)]
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

#[derive(PartialEq, Eq)]
pub enum Head {
    Detached { sha: String },
    Branch { name: String },
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
            (Head::Detached { sha: _ }, Head::Branch { name: _ }) => cmp::Ordering::Greater,
            (Head::Branch { name: _ }, Head::Detached { sha: _ }) => cmp::Ordering::Less,
            (Head::Branch { name: a }, Head::Branch { name: b }) => a.cmp(b),
        }
    }
}

// There are two modes, plain (compute-and-print-and-exit) and interactive (Elm loop)
// the interactive mode requires the following state to be present on the model
pub struct UIState {
    pub highlighted_line_index: usize,
    pub search_string: String,
    pub cursor_position: usize,
}

// the Non-interactive mode really just renders things on-demand
// and may not even need this type at all
pub enum Msg {
    Info(Vec<String>),
    Error { title: String, body: String },
}
