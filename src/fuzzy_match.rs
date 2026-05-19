const PREFIX_WINDOW: usize = 3;

pub fn fuzzy_match(search: &str, target: &str) -> usize {
    let matched_indices = find_sequential_indices(&search.to_lowercase(), &target.to_lowercase());

    let Some(matched_indices) = matched_indices else {
        return 0;
    };

    let prefix_bonus: usize = matched_indices
        .iter()
        .map(|&idx| PREFIX_WINDOW.saturating_sub(idx))
        .sum();

    let continuity_bonus: usize = matched_indices
        .windows(2)
        .filter(|w| (w[1] - w[0]) == 1)
        .count();

    1 + prefix_bonus + continuity_bonus
}

fn find_sequential_indices(needle: &str, haystack: &str) -> Option<Vec<usize>> {
    let mut indices: Vec<usize> = Vec::new();
    let mut start_index: usize = 0;

    for c in needle.chars() {
        let found = haystack[start_index..].find(c);
        match found {
            None => return None,
            Some(idx) => {
                indices.push(idx + start_index);
                start_index = idx + start_index + 1;
            }
        }
    }
    Some(indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    // find_sequential_indices

    #[test]
    fn indices_exact_match() {
        assert_eq!(find_sequential_indices("abc", "abc"), Some(vec![0, 1, 2]));
    }

    #[test]
    fn indices_skips_characters() {
        assert_eq!(find_sequential_indices("ac", "abc"), Some(vec![0, 2]));
    }

    #[test]
    fn indices_no_match() {
        assert_eq!(find_sequential_indices("xyz", "abc"), None);
    }

    #[test]
    fn indices_partial_match_returns_none() {
        assert_eq!(find_sequential_indices("abz", "abc"), None);
    }

    #[test]
    fn indices_repeated_chars_in_target() {
        assert_eq!(find_sequential_indices("aa", "banana"), Some(vec![1, 3]));
    }

    #[test]
    fn indices_match_at_end() {
        assert_eq!(find_sequential_indices("mn", "main"), Some(vec![0, 3]));
    }

    // fuzzy_match - no match

    #[test]
    fn no_match_returns_zero() {
        assert_eq!(fuzzy_match("xyz", "main"), 0);
    }

    // fuzzy_match - prefix bonus

    #[test]
    fn exact_prefix_scores_highest() {
        // "main" in "main": indices [0,1,2,3], prefix 3+2+1+0=6, continuity 3
        assert_eq!(fuzzy_match("main", "main"), 10);
    }

    #[test]
    fn later_match_has_lower_prefix_bonus() {
        // "main" in "xx-main": indices [3,4,5,6], prefix 0+0+0+0=0, continuity 3
        assert_eq!(fuzzy_match("main", "xx-main"), 4);
    }

    // fuzzy_match - continuity bonus

    #[test]
    fn consecutive_indices_get_continuity_bonus() {
        // "ab" in "abc": indices [0,1], prefix 3+2=5, continuity 1
        assert_eq!(fuzzy_match("ab", "abc"), 7);
    }

    #[test]
    fn non_consecutive_indices_get_no_continuity_bonus() {
        // "ac" in "abc": indices [0,2], prefix 3+1=4, continuity 0
        assert_eq!(fuzzy_match("ac", "abc"), 5);
    }

    // fuzzy_match - case insensitive

    #[test]
    fn case_insensitive() {
        assert_eq!(fuzzy_match("MAIN", "main"), fuzzy_match("main", "main"));
    }

    // fuzzy_match - ranking sanity checks

    #[test]
    fn exact_match_beats_scattered() {
        assert!(fuzzy_match("feat", "feature") > fuzzy_match("feat", "fix-everything"));
    }

    #[test]
    fn prefix_match_beats_suffix() {
        assert!(fuzzy_match("ma", "main") > fuzzy_match("ma", "xx-main"));
    }
}
