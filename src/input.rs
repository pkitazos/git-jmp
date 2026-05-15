pub fn next_word_boundary(text: &str, pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut i = pos;

    while i < chars.len() && is_breakpoint(chars[i]) {
        i += 1;
    }

    while i < chars.len() && !is_breakpoint(chars[i]) {
        i += 1;
    }

    i
}

pub fn prev_word_boundary(text: &str, pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut i = pos;

    while i > 0 && is_breakpoint(chars[i - 1]) {
        i -= 1;
    }

    while i > 0 && !is_breakpoint(chars[i - 1]) {
        i -= 1;
    }

    i
}

fn is_breakpoint(c: char) -> bool {
    matches!(c, ' ' | '-' | '/' | '.' | '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_word_boundary_at_end() {
        assert_eq!(next_word_boundary("hello", 5), 5);
    }

    #[test]
    fn test_next_word_boundary_empty() {
        assert_eq!(next_word_boundary("", 0), 0);
    }

    #[test]
    fn test_next_word_boundary_dash() {
        assert_eq!(next_word_boundary("foo-bar", 0), 3);
        assert_eq!(next_word_boundary("foo-bar", 3), 7);
    }

    #[test]
    fn test_next_word_boundary_slash() {
        assert_eq!(next_word_boundary("src/lib/main", 0), 3);
        assert_eq!(next_word_boundary("src/lib/main", 3), 7);
        assert_eq!(next_word_boundary("src/lib/main", 7), 12);
    }

    #[test]
    fn test_next_word_boundary_dot() {
        assert_eq!(next_word_boundary("foo.bar.baz", 0), 3);
        assert_eq!(next_word_boundary("foo.bar.baz", 3), 7);
        assert_eq!(next_word_boundary("foo.bar.baz", 7), 11);
    }

    #[test]
    fn test_next_word_boundary_underscore() {
        assert_eq!(next_word_boundary("snake_case_name", 0), 5);
        assert_eq!(next_word_boundary("snake_case_name", 5), 10);
        assert_eq!(next_word_boundary("snake_case_name", 10), 15);
    }

    #[test]
    fn test_next_word_boundary_consecutive_breakpoints() {
        assert_eq!(next_word_boundary("foo--bar", 0), 3);
        assert_eq!(next_word_boundary("foo--bar", 3), 8);
    }

    #[test]
    fn test_next_word_boundary_mixed_breakpoints() {
        assert_eq!(next_word_boundary("src/my_lib.rs", 0), 3);
        assert_eq!(next_word_boundary("src/my_lib.rs", 3), 6);
        assert_eq!(next_word_boundary("src/my_lib.rs", 6), 10);
        assert_eq!(next_word_boundary("src/my_lib.rs", 10), 13);
    }

    // prev_word_boundary tests

    #[test]
    fn test_prev_word_boundary_from_end() {
        assert_eq!(prev_word_boundary("hello world", 11), 6);
    }

    #[test]
    fn test_prev_word_boundary_from_space() {
        assert_eq!(prev_word_boundary("hello world", 5), 0);
    }

    #[test]
    fn test_prev_word_boundary_mid_word() {
        assert_eq!(prev_word_boundary("hello world", 8), 6);
    }

    #[test]
    fn test_prev_word_boundary_at_start() {
        assert_eq!(prev_word_boundary("hello", 0), 0);
    }

    #[test]
    fn test_prev_word_boundary_empty() {
        assert_eq!(prev_word_boundary("", 0), 0);
    }

    #[test]
    fn test_prev_word_boundary_dash() {
        assert_eq!(prev_word_boundary("foo-bar", 7), 4);
        assert_eq!(prev_word_boundary("foo-bar", 3), 0);
    }

    #[test]
    fn test_prev_word_boundary_slash() {
        assert_eq!(prev_word_boundary("src/lib/main", 12), 8);
        assert_eq!(prev_word_boundary("src/lib/main", 7), 4);
        assert_eq!(prev_word_boundary("src/lib/main", 3), 0);
    }

    #[test]
    fn test_prev_word_boundary_dot() {
        assert_eq!(prev_word_boundary("foo.bar.baz", 11), 8);
        assert_eq!(prev_word_boundary("foo.bar.baz", 7), 4);
    }

    #[test]
    fn test_prev_word_boundary_underscore() {
        assert_eq!(prev_word_boundary("snake_case_name", 15), 11);
        assert_eq!(prev_word_boundary("snake_case_name", 10), 6);
        assert_eq!(prev_word_boundary("snake_case_name", 5), 0);
    }

    #[test]
    fn test_prev_word_boundary_consecutive_breakpoints() {
        assert_eq!(prev_word_boundary("foo--bar", 8), 5);
        assert_eq!(prev_word_boundary("foo--bar", 3), 0);
    }

    #[test]
    fn test_prev_word_boundary_mixed_breakpoints() {
        assert_eq!(prev_word_boundary("src/my_lib.rs", 13), 11);
        assert_eq!(prev_word_boundary("src/my_lib.rs", 10), 7);
        assert_eq!(prev_word_boundary("src/my_lib.rs", 6), 4);
        assert_eq!(prev_word_boundary("src/my_lib.rs", 3), 0);
    }

    #[test]
    fn test_next_prev_roundtrip_from_word_starts() {
        // prev(next(start)) == start when starting from a word-start boundary
        let word_end = next_word_boundary("src/my_lib.rs", 4);
        let back_to_start = prev_word_boundary("src/my_lib.rs", word_end);
        assert_eq!(back_to_start, 4);
    }

    #[test]
    fn test_prev_next_roundtrip_from_word_ends() {
        // next(prev(end)) == end when starting from a word-end boundary
        let word_start = prev_word_boundary("src/my_lib.rs", 10);
        let back_to_end = next_word_boundary("src/my_lib.rs", word_start);
        assert_eq!(back_to_end, 10);
    }
}
