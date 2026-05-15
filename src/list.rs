use std::{collections::HashSet, path::Path};

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
    worktrees
        .iter()
        .filter_map(|w| match &w.head {
            Head::Detached { .. } => None,
            Head::Branch(branch) => Some(branch.name.as_str()),
        })
        .collect()
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
        .iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    fn branch(name: &str, last_switch: u64) -> Branch {
        Branch {
            name: name.to_string(),
            last_switch,
        }
    }

    fn worktree(dir: &str, branch_name: &str, last_switch: u64) -> Worktree {
        Worktree {
            dir: PathBuf::from(dir),
            head: Head::Branch(branch(branch_name, last_switch)),
        }
    }

    // worktree_branch_names

    #[test]
    fn worktree_branch_names_extracts_labels() {
        let ws = vec![worktree("/a", "main", 0), worktree("/b", "dev", 0)];
        let names = worktree_branch_names(&ws);
        assert!(names.contains("main"));
        assert!(names.contains("dev"));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn worktree_branch_names_includes_detached() {
        let ws = vec![Worktree {
            dir: PathBuf::from("/a"),
            head: Head::Detached {
                sha: "abc123".to_string(),
            },
        }];
        let names = worktree_branch_names(&ws);
        assert!(!names.contains("abc123"));
    }

    // prep_available_branches

    #[test]
    fn prep_available_excludes_checked_out_branches() {
        let bs = vec![branch("main", 10), branch("dev", 5), branch("feature", 1)];
        let ws = vec![worktree("/a", "main", 10)];
        let available = prep_available_branches(&bs, &ws);
        let names: Vec<&str> = available.iter().map(|b| b.name.as_str()).collect();
        assert!(!names.contains(&"main"));
        assert!(names.contains(&"dev"));
        assert!(names.contains(&"feature"));
    }

    #[test]
    fn prep_available_sorted_by_last_switch_desc() {
        let branches = vec![branch("old", 1), branch("recent", 10), branch("mid", 5)];
        let available = prep_available_branches(&branches, &[]);
        let names: Vec<&str> = available.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(names, vec!["recent", "mid", "old"]);
    }

    #[test]
    fn prep_available_empty_when_all_checked_out() {
        let branches = vec![branch("main", 10)];
        let ws = vec![worktree("/a", "main", 10)];
        let available = prep_available_branches(&branches, &ws);
        assert!(available.is_empty());
    }

    // prep_available_worktrees

    #[test]
    fn prep_available_worktrees_excludes_active() {
        let ws = vec![
            worktree("/active", "main", 10),
            worktree("/other", "dev", 5),
        ];
        let available = prep_available_worktrees(&ws, Path::new("/active"));
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].dir, PathBuf::from("/other"));
    }

    #[test]
    fn prep_available_worktrees_empty_when_only_active() {
        let ws = vec![worktree("/active", "main", 10)];
        let available = prep_available_worktrees(&ws, Path::new("/active"));
        assert!(available.is_empty());
    }

    // generate_ranked_list

    #[test]
    fn ranked_list_filters_non_matching() {
        let branches = vec![branch("main", 10), branch("feature", 5)];
        let result = generate_ranked_list(&branches, &[], "xyz");
        assert!(result.available.is_empty());
    }

    #[test]
    fn ranked_list_best_match_first() {
        let branches = vec![branch("fix-something", 5), branch("main", 10)];
        let result = generate_ranked_list(&branches, &[], "main");
        assert_eq!(result.available[0].name, "main");
    }

    #[test]
    fn ranked_list_includes_worktrees() {
        let ws = vec![worktree("/a", "main", 10), worktree("/b", "dev", 5)];
        let result = generate_ranked_list(&[], &ws, "main");
        assert_eq!(result.worktrees.len(), 1);
        assert_eq!(result.worktrees[0].head.label(), "main");
    }
}
