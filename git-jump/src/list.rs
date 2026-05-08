use std::{collections::HashSet, iter};

use crate::{
    fuzzy_match::fuzzy_match,
    types::{Branch, Head, RankedSearchList, Worktree},
};

#[derive(PartialEq, Eq, PartialOrd)]
struct MatchRecord<T> {
    match_score: usize,
    item: T,
}

pub fn prep_available_branches(branches: &[Branch], worktrees: &[Worktree]) -> Vec<Branch> {
    let mut available_branches: Vec<Branch> = {
        // move checkout out into a separate scope so that it's dropped
        // after we're done constructing the hashset
        let checked_out: HashSet<&str> = worktrees.iter().map(|w| w.head.label()).collect();

        branches
            .iter()
            .filter(|b| !checked_out.contains(b.name.as_str()))
            .cloned()
            .collect()
    };

    available_branches.sort();
    available_branches
}

pub fn generate_ranked_list(
    head: &Head,
    branches: &[Branch],
    worktrees: &[Worktree],
    search_string: &str,
) -> RankedSearchList {
    let mut available: Vec<MatchRecord<Head>> = iter::once(MatchRecord {
        match_score: fuzzy_match(search_string, head.label()),
        item: head.clone(),
    })
    .chain(branches.iter().map(|b| MatchRecord {
        match_score: fuzzy_match(search_string, &b.name),
        item: b.to_owned().into_head(),
    }))
    .collect();

    available.sort_by(|a, b| {
        b.match_score
            .cmp(&a.match_score)
            .then_with(|| b.item.last_switched().cmp(&a.item.last_switched()))
            .then_with(|| a.item.label().cmp(&b.item.label()))
    });

    let available: Vec<Head> = available
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
