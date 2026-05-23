use std::{
    cmp,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Branch {
    pub name: String,
    pub last_switch: u64,
}

impl Branch {
    pub fn is_head(&self, head: &Head) -> bool {
        self.name == head.label()
    }

    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            last_switch: 0,
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
