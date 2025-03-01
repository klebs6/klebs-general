// ---------------- [ File: src/dissect_use_statement.rs ]
crate::ix!();

/// Attempt to dissect e.g. "pub(crate) use std::collections::HashMap;"
/// but ensure the substring "use " is either at the start
/// or preceded by whitespace, so "pubuse foo;" won't match.
pub fn dissect_use_statement(line: &str) -> Option<(String, String, String)> {
    let trimmed = line.trim();
    // First, find the substring "use ":
    let use_idx = trimmed.find("use ")?;
    // Next, ensure that the character before "use " is either
    // the beginning of the line or a whitespace char.
    if use_idx > 0 {
        let prev_char = trimmed.chars().nth(use_idx - 1)?;
        if !prev_char.is_whitespace() {
            // e.g. "pubuse foo;" => preceding char is 'b', so we bail out
            return None;
        }
    }
    // If we pass that check, proceed as before:
    let prefix = &trimmed[..use_idx].trim();
    let after_use = &trimmed[use_idx + 4..];
    let after_use = after_use.trim_end_matches(';').trim();
    Some((prefix.to_string(), "use".to_string(), after_use.to_string()))
}

#[cfg(test)]
mod test_dissect_use_statement {
    use super::*;

    /// 1) If the line does not contain "use " at all => return None.
    #[traced_test]
    fn test_no_use_substring_returns_none() {
        let input = "some random line pub(crate) x std::foo;";
        let result = dissect_use_statement(input);
        assert!(result.is_none(), "Expected None if 'use ' is absent");
    }

    /// 2) A basic use line with a prefix (e.g. "pub(crate)").
    #[traced_test]
    fn test_prefix_pub_crate() {
        let input = "pub(crate) use std::collections::HashMap;";
        let result = dissect_use_statement(input);
        assert!(result.is_some(), "Expected Some(...)");
        let (prefix, keyword, remainder) = result.unwrap();
        assert_eq!(prefix, "pub(crate)", "Should capture prefix as 'pub(crate)'");
        assert_eq!(keyword, "use", "Should capture 'use'");
        assert_eq!(remainder, "std::collections::HashMap", "Should parse the path minus trailing semicolon");
    }

    /// 3) No prefix, e.g. "use std::io;" => prefix is "".
    #[traced_test]
    fn test_no_prefix() {
        let input = "use std::io;";
        let result = dissect_use_statement(input);
        assert!(result.is_some());
        let (prefix, keyword, remainder) = result.unwrap();
        assert_eq!(prefix, "", "No prefix => empty string");
        assert_eq!(keyword, "use");
        assert_eq!(remainder, "std::io");
    }

    /// 4) Leading/trailing whitespace around the line => ensure they're trimmed.
    #[traced_test]
    fn test_leading_trailing_spaces() {
        let input = "   use   crate::foo::bar   ;   ";
        let result = dissect_use_statement(input);
        assert!(result.is_some());
        let (prefix, keyword, remainder) = result.unwrap();
        assert_eq!(prefix, "", "No explicit prefix");
        assert_eq!(keyword, "use");
        assert_eq!(remainder, "crate::foo::bar", "Should trim internal spaces around the path");
    }

    /// 5) A prefix "pub" (no parentheses), no semicolon => that might still parse, ignoring the missing semicolon.
    #[traced_test]
    fn test_pub_prefix_no_semicolon() {
        let input = "pub use crate::something";
        let result = dissect_use_statement(input);
        assert!(result.is_some());
        let (prefix, keyword, remainder) = result.unwrap();
        assert_eq!(prefix, "pub");
        assert_eq!(keyword, "use");
        assert_eq!(remainder, "crate::something", "No semicolon => still captured as remainder");
    }

    /// 6) Multiple spaces after 'use' => we skip them, everything after is the remainder.
    #[traced_test]
    fn test_multiple_spaces_after_use() {
        let input = "pub use    foo::bar;";
        let result = dissect_use_statement(input);
        assert!(result.is_some());
        let (prefix, keyword, remainder) = result.unwrap();
        assert_eq!(prefix, "pub");
        assert_eq!(keyword, "use");
        assert_eq!(remainder, "foo::bar");
    }

    /// 7) Minimal line: "use foo"
    #[traced_test]
    fn test_minimal_line_no_semicolon() {
        let input = "use foo";
        let result = dissect_use_statement(input);
        assert!(result.is_some());
        let (prefix, keyword, remainder) = result.unwrap();
        assert_eq!(prefix, "");
        assert_eq!(keyword, "use");
        assert_eq!(remainder, "foo");
    }

    /// 8) A line with random leading text but containing "use " in the middle might return None
    ///    because we assume the statement starts with the prefix up to "use". But let's confirm 
    ///    how the code behaves: it will do .find("use "), so if the string is "bla use something", 
    ///    we parse prefix="bla", remainder="something". That's still Some(...).
    ///    So let's test that scenario.
    #[traced_test]
    fn test_random_leading_text_in_prefix() {
        let input = "bla use something;";
        let result = dissect_use_statement(input);
        assert!(result.is_some(), "We do find 'use ' => we parse up to that as prefix");
        let (prefix, keyword, remainder) = result.unwrap();
        assert_eq!(prefix, "bla", "Everything before 'use ' is the prefix");
        assert_eq!(keyword, "use");
        assert_eq!(remainder, "something");
    }

    /// 9) Empty input => obviously no "use " => returns None
    #[traced_test]
    fn test_empty_input() {
        let input = "";
        let result = dissect_use_statement(input);
        assert!(result.is_none());
    }

    /// 10) If "use" is present but not followed by a space, e.g. "pubuse foo" => should not parse
    #[traced_test]
    fn test_use_not_followed_by_space() {
        let input = "pubuse foo;";
        let result = dissect_use_statement(input);
        // We want None, because there's no true " use " substring with whitespace or start-of-line before "use".
        assert!(
            result.is_none(),
            "Expected None if 'pubuse' is used, since there's no whitespace or start boundary before 'use '"
        );
    }
}
