// ---------------- [ File: workspacer-consolidate/src/leading_spaces.rs ]
crate::ix!();

/// Returns how many leading space characters are at the start of `line`.
pub fn leading_spaces(line: &str) -> usize {
    let mut count = 0;
    for c in line.chars() {
        if c == ' ' {
            count += 1;
        } else {
            break;
        }
    }
    count
}

#[cfg(test)]
mod test_leading_spaces {
    use super::leading_spaces;

    /// Verifies that leading_spaces returns 0 for an empty string.
    #[test]
    fn test_leading_spaces_empty_string() {
        let line = "";
        let result = leading_spaces(line);
        assert_eq!(result, 0, "Empty string should have 0 leading spaces");
    }

    /// Verifies that leading_spaces returns 0 when the first character is non-space.
    #[test]
    fn test_leading_spaces_no_leading_spaces() {
        let line = "hello world";
        let result = leading_spaces(line);
        assert_eq!(result, 0, "No leading spaces, so result should be 0");
    }

    /// Verifies that leading_spaces counts all consecutive spaces up to the first non-space.
    #[test]
    fn test_leading_spaces_mixed_content() {
        let line = "   some text";
        let result = leading_spaces(line);
        assert_eq!(result, 3, "Expected 3 leading spaces");
    }

    /// Verifies that leading_spaces returns the length of the entire string if it is all spaces.
    #[test]
    fn test_leading_spaces_all_spaces() {
        let line = "      ";
        let result = leading_spaces(line);
        assert_eq!(
            result,
            line.len(),
            "All spaces => count should match length of the string"
        );
    }
}
