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
