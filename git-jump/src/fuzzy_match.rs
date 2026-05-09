fn find_sequential_indices(search: &str, target: &str) -> Option<Vec<usize>> {
    let mut indices: Vec<usize> = Vec::new();
    let mut start_index: usize = 0;

    for c in search.chars() {
        let found = target[start_index..].find(c);
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

const PREFIX_WINDOW: usize = 3;

pub fn fuzzy_match(search: &str, target: &str) -> usize {
    let matched_indices = find_sequential_indices(&search.to_lowercase(), &target.to_lowercase());

    let Some(matched_indices) = matched_indices else {
        return 0;
    };

    let prefix_bonus: usize = matched_indices
        .iter()
        .map(|&idx| (PREFIX_WINDOW.saturating_sub(idx)))
        .sum();

    let continuity_bonus: usize = matched_indices
        .windows(2)
        .map(|w| (w[1] - w[0]) == 1)
        .count();

    1 + prefix_bonus + continuity_bonus
}
