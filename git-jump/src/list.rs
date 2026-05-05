use std::collections::HashSet;

use crate::{
    fuzzy_match::fuzzy_match,
    types::{Branch, Head, Worktree},
    utils::now,
};

// the interactive mode renders the available branches using the following rules:
// - render the currently checked out branch / hash first
// - render all the rest of the branches (nothing detached) that are not checked out in worktrees
// - render all the rest of the branches that *are* checked out in other linked worktrees

pub enum SearchList {
    Idle {
        head: Head,
        available_branches: Vec<Branch>,
        in_worktrees: Vec<Worktree>,
    },
    InSearch {
        available: Vec<Head>,
        worktrees: Vec<Worktree>,
    },
}

#[derive(PartialEq, Eq, PartialOrd)]
struct VisitedHead {
    head: Head,
    last_switch: u64,
}

#[derive(PartialEq, Eq, PartialOrd)]
struct MatchRecord<T> {
    match_score: usize,
    item: T,
}

pub fn generate_list(
    current_head: Head,
    branches: Vec<Branch>,
    worktrees: Vec<Worktree>,
    search_string: &str,
) -> SearchList {
    let mut available_branches: Vec<Branch> = {
        // move checkout out into a separate scope so that it's dropped
        // after we're done constructing the hashset
        let checked_out: HashSet<&str> = worktrees.iter().map(|w| w.head.label()).collect();

        branches
            .into_iter()
            .filter(|b| !checked_out.contains(b.name.as_str()))
            .collect()
    };

    available_branches.sort();

    if search_string.is_empty() {
        let mut in_worktrees = worktrees;
        in_worktrees.sort();

        return SearchList::Idle {
            head: current_head,
            available_branches,
            in_worktrees,
        };
    }

    let mut available: Vec<MatchRecord<VisitedHead>> = available_branches
        .into_iter()
        .map(|b| MatchRecord {
            match_score: fuzzy_match(search_string, &b.name),
            item: VisitedHead {
                head: Head::Branch { name: b.name },
                last_switch: b.last_switch,
            },
        })
        .collect();

    let now = now();

    let head_match_score = match &current_head {
        Head::Detached { sha } => fuzzy_match(search_string, sha),
        Head::Branch { name } => fuzzy_match(search_string, name),
    };

    available.push(MatchRecord {
        match_score: head_match_score,
        item: VisitedHead {
            head: current_head,
            last_switch: now,
        },
    });

    available.sort_by(|a, b| {
        b.match_score
            .cmp(&a.match_score)
            .then_with(|| b.item.last_switch.cmp(&a.item.last_switch))
            .then_with(|| a.item.head.cmp(&b.item.head))
    });

    let available: Vec<Head> = available
        .into_iter()
        .filter(|r| r.match_score > 0)
        .map(|r| r.item.head)
        .collect();

    let mut worktrees: Vec<MatchRecord<Worktree>> = worktrees
        .into_iter()
        .map(|w| MatchRecord {
            match_score: fuzzy_match(search_string, w.head.label()),
            item: w,
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

    SearchList::InSearch {
        available,
        worktrees,
    }
}
