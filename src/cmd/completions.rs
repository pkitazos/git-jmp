use clap_complete::CompletionCandidate;
use std::ffi::OsStr;

use crate::git::read_raw_git_branches;

pub fn branch_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    let branches = read_raw_git_branches().unwrap_or_default();

    let Some(current) = current.to_str() else {
        return branches.iter().map(CompletionCandidate::new).collect();
    };

    branches
        .iter()
        .filter(|b| b.starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}
