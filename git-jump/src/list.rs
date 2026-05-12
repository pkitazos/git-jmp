use std::{collections::HashSet, path::Path};

use crate::{
    fuzzy_match::fuzzy_match,
    types::{Branch, RankedSearchList, Worktree},
};

#[derive(PartialEq, Eq, PartialOrd)]
struct MatchRecord<T> {
    match_score: usize,
    item: T,
}

pub fn prep_available_branches(branches: &[Branch], worktrees: &[Worktree]) -> Vec<Branch> {
    let checked_out: HashSet<&str> = worktree_branch_names(worktrees);

    let mut available_branches: Vec<Branch> = branches
        .iter()
        .filter(|b| !checked_out.contains(b.name.as_str()))
        .cloned()
        .collect();

    available_branches.sort();
    available_branches
}

pub fn worktree_branch_names(worktrees: &[Worktree]) -> HashSet<&str> {
    worktrees.iter().map(|w| w.head.label()).collect()
}

pub fn prep_available_worktrees(
    worktrees: &[Worktree],
    active_worktree_dir: &Path,
) -> Vec<Worktree> {
    let mut available_worktrees: Vec<Worktree> = worktrees
        .iter()
        .filter(|w| w.dir != active_worktree_dir)
        .cloned()
        .collect();

    available_worktrees.sort();
    available_worktrees
}

pub fn generate_ranked_list(
    branches: &[Branch],
    worktrees: &[Worktree],
    search_string: &str,
) -> RankedSearchList {
    let mut available: Vec<MatchRecord<Branch>> = branches
        .iter()
        .map(|b| MatchRecord {
            match_score: fuzzy_match(search_string, &b.name),
            item: b.to_owned(),
        })
        .collect();

    available.sort_by(|a, b| {
        b.match_score
            .cmp(&a.match_score)
            .then_with(|| b.item.cmp(&a.item))
    });

    let available: Vec<Branch> = available
        .into_iter()
        .filter(|r| r.match_score > 0)
        .map(|r| r.item)
        .collect();

    let mut worktrees: Vec<MatchRecord<Worktree>> = worktrees
        .into_iter()
        .map(|w| MatchRecord {
            match_score: fuzzy_match(search_string, w.head.label()),
            item: w.clone(),
        })
        .collect();

    worktrees.sort_by(|a, b| {
        a.match_score
            .cmp(&b.match_score)
            .reverse()
            .then_with(|| a.item.head.cmp(&b.item.head))
    });

    let worktrees: Vec<Worktree> = worktrees
        .into_iter()
        .filter(|r| r.match_score > 0)
        .map(|r| r.item)
        .collect();

    RankedSearchList {
        available,
        worktrees,
    }
}
