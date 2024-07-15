#[inline]
pub fn bisect(text: &str) -> (&str, &str) {
    if text.is_empty() {
        return ("", "");
    }
    
    let char_count = text.chars().count();
    let midpoint = (char_count + 1) / 2;  // Ensures the middle character goes to the first half if odd length

    let mut char_indices = text.char_indices();
    let (split_index, _) = char_indices.nth(midpoint - 1).unwrap_or((text.len(), '\0'));

    (&text[..split_index + text.chars().nth(midpoint - 1).unwrap().len_utf8()], &text[split_index + text.chars().nth(midpoint - 1).unwrap().len_utf8()..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_even_length_ascii() {
        let text = "abcdef";
        let (first_half, second_half) = bisect(text);
        assert_eq!(first_half, "abc");
        assert_eq!(second_half, "def");
    }

    #[test]
    fn test_odd_length_ascii() {
        let text = "abcde";
        let (first_half, second_half) = bisect(text);
        assert_eq!(first_half, "abc");
        assert_eq!(second_half, "de");
    }

    #[test]
    fn test_empty_string() {
        let text = "";
        let (first_half, second_half) = bisect(text);
        assert_eq!(first_half, "");
        assert_eq!(second_half, "");
    }

    #[test]
    fn test_single_character() {
        let text = "a";
        let (first_half, second_half) = bisect(text);
        assert_eq!(first_half, "a");
        assert_eq!(second_half, "");
    }

    #[test]
    fn test_multi_byte_characters() {
        let text = "a😊b😊c";
        let (first_half, second_half) = bisect(text);
        assert_eq!(first_half, "a😊b");
        assert_eq!(second_half, "😊c");
    }

    #[test]
    fn test_multi_byte_characters_split_boundary() {
        let text = "a😊bc😊";
        let (first_half, second_half) = bisect(text);
        assert_eq!(first_half, "a😊b");
        assert_eq!(second_half, "c😊");
    }
}
