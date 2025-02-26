// ---------------- [ File: src/make_top_block_macro_lines.rs ]
crate::ix!();

/// Creates lines for the top block. For example:
///
/// ```text
/// // ---------------- [ File: src/lib.rs ]
/// x!{crate_handle}
/// x!{foo}
/// x!{bar}
/// ```
pub fn make_top_block_macro_lines(stems: &[String]) -> String {
    let mut lines = vec![];
    lines.push("// ---------------- [ File: src/lib.rs ]".to_string());
    for st in stems {
        lines.push(format!("x!{{{}}}", st));
    }
    lines.join("\n")
}

#[cfg(test)]
mod test_make_top_block_macro_lines {
    use super::*;

    /// 1) If stems is empty => we only get the file marker line
    #[test]
    fn test_empty_stems() {
        let stems: Vec<String> = vec![];
        let result = make_top_block_macro_lines(&stems);
        let expected = "// ---------------- [ File: src/lib.rs ]";
        assert_eq!(result, expected, "Should only show the file marker with no x! lines");
    }

    /// 2) If there's a single stem => we expect two lines
    #[test]
    fn test_single_stem() {
        let stems = vec!["my_stem".to_string()];
        let result = make_top_block_macro_lines(&stems);
        let expected = format!(
            "{}\n{}",
            "// ---------------- [ File: src/lib.rs ]",
            "x!{my_stem}"
        );
        assert_eq!(result, expected);
    }

    /// 3) If there are multiple stems => we build a line for each
    #[test]
    fn test_multiple_stems() {
        let stems = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let result = make_top_block_macro_lines(&stems);

        // Lines:
        // // ---------------- [ File: src/lib.rs ]
        // x!{one}
        // x!{two}
        // x!{three}
        let expected = [
            "// ---------------- [ File: src/lib.rs ]",
            "x!{one}",
            "x!{two}",
            "x!{three}",
        ]
        .join("\n");

        assert_eq!(result, expected);
    }

    /// 4) If stems contain something with special characters, we ensure it's inserted literally.
    #[test]
    fn test_special_characters_in_stems() {
        let stems = vec!["foo_bar-baz".to_string(), "stuff123".to_string()];
        let result = make_top_block_macro_lines(&stems);

        let expected = [
            "// ---------------- [ File: src/lib.rs ]",
            "x!{foo_bar-baz}",
            "x!{stuff123}",
        ]
        .join("\n");

        assert_eq!(result, expected);
    }

    /// 5) If stems contain an empty string, we produce `x!{}` on a line
    #[test]
    fn test_empty_string_stem() {
        let stems = vec!["".to_string(), "normal".to_string()];
        let result = make_top_block_macro_lines(&stems);
        let expected = [
            "// ---------------- [ File: src/lib.rs ]",
            "x!{}",
            "x!{normal}",
        ]
        .join("\n");

        assert_eq!(result, expected);
    }

    /// 6) Verify that the function doesn’t add extra trailing newlines: 
    ///    It's a single string joined by "\n", no trailing newline beyond the last line.
    #[test]
    fn test_no_trailing_newline() {
        let stems = vec!["alpha".to_string()];
        let result = make_top_block_macro_lines(&stems);
        // We want to confirm the final string does NOT end with "\n".
        assert!(!result.ends_with('\n'), "Should not have an extra trailing newline");
    }

    /// 7) Large number of stems => we just produce that many lines (not a performance test, but just for sanity).
    #[test]
    fn test_many_stems() {
        let stems: Vec<String> = (1..=5).map(|i| format!("item_{i}")).collect();
        let result = make_top_block_macro_lines(&stems);
        let expected = [
            "// ---------------- [ File: src/lib.rs ]",
            "x!{item_1}",
            "x!{item_2}",
            "x!{item_3}",
            "x!{item_4}",
            "x!{item_5}",
        ]
        .join("\n");
        assert_eq!(result, expected);
    }
}
