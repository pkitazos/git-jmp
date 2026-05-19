use std::{
    cmp,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

pub struct Model {
    pub main_worktree: MainWorktree,
    pub active_worktree: PathBuf,

    pub branches: Vec<Branch>,
    pub worktrees: Vec<Worktree>,
}

pub struct MainWorktree {
    pub project_root_dir: PathBuf,
}

/// The name of the hidden directory created within the target Git repository
/// to store jump-related metadata.
pub const JUMP_FOLDER: &str = ".jump";

/// The name of the JSON file where branch usage history and timestamps are saved.
pub const DATA_FILE: &str = "data.json";

impl MainWorktree {
    pub fn root(&self) -> &Path {
        &self.project_root_dir
    }

    pub fn git_dir(&self) -> PathBuf {
        self.project_root_dir.join(".git")
    }

    pub fn jump_dir(&self) -> PathBuf {
        self.root().join(JUMP_FOLDER)
    }

    pub fn data_file(&self) -> PathBuf {
        self.jump_dir().join(DATA_FILE)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Branch {
    pub name: String,
    pub last_switch: u64,
}

impl Branch {
    pub fn is_head(&self, head: &Head) -> bool {
        self.name == head.label()
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Head {
    Detached { sha: String },
    Branch(Branch),
}

impl Head {
    pub fn label(&self) -> &str {
        match self {
            Head::Detached { sha } => sha,
            Head::Branch(b) => &b.name,
        }
    }

    pub fn into_label(self) -> String {
        match self {
            Head::Detached { sha } => sha,
            Head::Branch(b) => b.name,
        }
    }

    pub fn last_switched(&self) -> u64 {
        match self {
            Head::Detached { .. } => now(),
            Head::Branch(b) => b.last_switch.to_owned(),
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
            (Head::Branch(a), Head::Branch(b)) => a
                .last_switch
                .cmp(&b.last_switch)
                .then_with(|| a.name.cmp(&b.name)),
        }
    }
}

pub fn now() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => u64::MAX,
    }
}

#[derive(Error, Debug)]
pub enum GitJumpError {
    #[error("Failed to create branch")]
    BranchCreation(#[source] anyhow::Error),

    #[error("Failed to rename branch")]
    BranchRenaming(#[source] anyhow::Error),

    #[error("{target} does not match any branch")]
    NoMatch { target: String },

    #[error("Failed to switch branch")]
    SwitchFailed(#[source] anyhow::Error),

    #[error("Can't rename: HEAD is detached, specify the branch explicitly")]
    DetachedHead,

    #[error("")]
    SilentExit,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
