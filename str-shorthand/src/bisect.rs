pub trait Bisect {
    type Output;
    fn bisect(&self) -> Self::Output;
}

impl<'a> Bisect for &'a str {

    type Output = (&'a str, &'a str);

    #[inline]
    fn bisect(&self) -> (&'a str, &'a str) {
        if self.is_empty() {
            return ("", "");
        }
        
        let char_count = self.chars().count();
        let midpoint = (char_count + 1) / 2;  // Ensures the middle character goes to the first half if odd length

        let mut char_indices = self.char_indices();
        let (split_index, _) = char_indices.nth(midpoint - 1).unwrap_or((self.len(), '\0'));

        (&self[..split_index + self.chars().nth(midpoint - 1).unwrap().len_utf8()], &self[split_index + self.chars().nth(midpoint - 1).unwrap().len_utf8()..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_even_length_ascii() {
        let text = "abcdef";
        let (first_half, second_half) = text.bisect();
        assert_eq!(first_half, "abc");
        assert_eq!(second_half, "def");
    }

    #[test]
    fn test_odd_length_ascii() {
        let text = "abcde";
        let (first_half, second_half) = text.bisect();
        assert_eq!(first_half, "abc");
        assert_eq!(second_half, "de");
    }

    #[test]
    fn test_empty_string() {
        let text = "";
        let (first_half, second_half) = text.bisect();
        assert_eq!(first_half, "");
        assert_eq!(second_half, "");
    }

    #[test]
    fn test_single_character() {
        let text = "a";
        let (first_half, second_half) = text.bisect();
        assert_eq!(first_half, "a");
        assert_eq!(second_half, "");
    }

    #[test]
    fn test_multi_byte_characters() {
        let text = "a😊b😊c";
        let (first_half, second_half) = text.bisect();
        assert_eq!(first_half, "a😊b");
        assert_eq!(second_half, "😊c");
    }

    #[test]
    fn test_multi_byte_characters_split_boundary() {
        let text = "a😊bc😊";
        let (first_half, second_half) = text.bisect();
        assert_eq!(first_half, "a😊b");
        assert_eq!(second_half, "c😊");
    }
}
