pub enum BracketType {
    Curly,
    Square,
    Paren,
}

pub fn find_matching_bracket(text: &str, start: usize, bracket_type: BracketType) -> Option<usize> {
    let (open, close) = match bracket_type {
        BracketType::Curly => ('{', '}'),
        BracketType::Square => ('[', ']'),
        BracketType::Paren => ('(', ')'),
    };

    // Ensure the start index is within bounds and is an opening bracket
    let mut char_indices = text.char_indices();
    let mut found_start = false;

    // Find the starting character index
    for (i, c) in char_indices.by_ref() {
        if i == start && c == open {
            found_start = true;
            break;
        }
    }

    if !found_start {
        return None;
    }

    let mut bracket_count = 1;

    for (i, c) in char_indices {
        match c {
            c if c == open => bracket_count += 1,
            c if c == close => {
                bracket_count -= 1;
                if bracket_count == 0 {
                    return Some(i);
                }
            },
            _ => {}
        }
    }
    None
}

pub fn find_matching_curly_brace(text: &str, start: usize) -> Option<usize> {
    find_matching_bracket(text, start, BracketType::Curly)
}

pub fn find_matching_square_bracket(text: &str, start: usize) -> Option<usize> {
    find_matching_bracket(text, start, BracketType::Square)
}

pub fn find_matching_paren(text: &str, start: usize) -> Option<usize> {
    find_matching_bracket(text, start, BracketType::Paren)
}

#[cfg(test)]
mod test_find_matching_bracket {

    use super::*;

    #[test]
    fn test_curly_braces() {
        let text = "{content}";
        assert_eq!(find_matching_curly_brace(text, 0), Some(8));
    }

    #[test]
    fn test_square_brackets() {
        let text = "[content]";
        assert_eq!(find_matching_square_bracket(text, 0), Some(8));
    }

    #[test]
    fn test_parentheses() {
        let text = "(content)";
        assert_eq!(find_matching_paren(text, 0), Some(8));
    }

    #[test]
    fn test_nested_brackets() {
        let text = "{[()][]}";
        assert_eq!(find_matching_curly_brace(text, 0), Some(7));
        assert_eq!(find_matching_square_bracket(text, 1), Some(4));
        assert_eq!(find_matching_paren(text, 2), Some(3));
    }

    #[test]
    fn test_no_matching_bracket() {
        let text = "{[()]}[";
        assert_eq!(find_matching_square_bracket(text, 6), None);
    }

    #[test]
    fn test_large_input() {
        let mut text = "{".to_string();
        text.push_str(&"a".repeat(10000));  // Long sequence of 'a's
        text.push_str("}");
        assert_eq!(find_matching_curly_brace(&text, 0), Some(10001));
    }

    #[test]
    fn test_unicode_characters() {
        let text = "{内容[嵌套(😊)层次]}";

        // Find the indices for the brackets
        let start_square = text.char_indices().nth(3).map(|(i, _)| i).unwrap();
        let end_square   = text.char_indices().nth(11).map(|(i, _)| i).unwrap();
        let start_paren  = text.char_indices().nth(6).map(|(i, _)| i).unwrap();
        let end_paren    = text.char_indices().nth(8).map(|(i, _)| i).unwrap();

        assert_eq!(find_matching_curly_brace(text, 0), Some(text.len() - 1));
        assert_eq!(find_matching_square_bracket(text, start_square), Some(end_square));
        assert_eq!(find_matching_paren(text, start_paren), Some(end_paren));
    }

    #[test]
    fn test_edge_cases() {
        // Empty string
        assert_eq!(find_matching_curly_brace("", 0), None);

        // Start index beyond string length
        let text = "{content}";
        assert_eq!(find_matching_curly_brace(text, text.len()), None);

        // Start index way beyond string length
        assert_eq!(find_matching_curly_brace(text, text.len() + 100), None);
    }
}
