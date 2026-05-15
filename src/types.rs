use crate::{app::InteractiveApp, system::InitData, utils::now};
use anyhow::Result;
use std::{
    cmp,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub struct Model {
    pub main_worktree: MainWorktree,
    pub active_worktree: PathBuf,
    pub modifier_key: ModifierKey,
    pub columns: usize,
    pub rows: usize,
    pub max_rows: usize,
    pub branches: Vec<Branch>,
    pub worktrees: Vec<Worktree>,
    pub interactive_state: Option<InteractiveApp>,
}

impl Model {
    pub fn new(data: InitData) -> Result<Model> {
        let (columns, rows) = crossterm::terminal::size()?;

        Ok(Model {
            main_worktree: data.main_worktree,
            active_worktree: data.active_worktree,
            columns: columns as usize,
            rows: rows as usize,
            max_rows: rows as usize,
            branches: data.branches,
            worktrees: data.worktrees,
            modifier_key: if std::env::consts::OS == "macos" {
                ModifierKey::Option
            } else {
                ModifierKey::Alt
            },
            interactive_state: None,
        })
    }
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

pub enum ModifierKey {
    Alt,
    Option,
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

pub fn get_active_worktree(worktrees: &Vec<Worktree>, active_worktree_dir: &Path) -> Worktree {
    worktrees
        .iter()
        .find(|w| w.dir.eq(&active_worktree_dir))
        .unwrap()
        .clone()
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

#[derive(Clone)]
pub struct RankedSearchList {
    /// all branches you can jump to that match the search input
    pub available: Vec<Branch>,
    /// all worktrees that match the search input
    pub worktrees: Vec<Worktree>,
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

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub enum BranchDeleteResult {
    Deleted(String),        // branch name
    Failed(String, String), // branch name, reason
}
