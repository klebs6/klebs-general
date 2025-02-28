crate::ix!();

/// A small helper for pushing a `TopBlockMacro` (leading comments + `x!{stem}`),
/// ensuring each part ends in exactly one `\n` if needed. 
/// 
/// **Key points**:
///  - If `buffer` is non-empty and doesn't end with `\n`, push one before anything else.
///  - Then write the macro's leading comments (if any).
///  - If those comments do **not** end in `\n`, push one.
///  - Finally, append `x!{stem}`.
pub fn push_macro_with_comments(buffer: &mut String, mac: &TopBlockMacro) {
    // 1) Ensure buffer is separated from new content by one newline.
    maybe_ensure_newline(buffer);

    // 2) If there are leading comments, push them, ensuring they end with newline.
    if let Some(comments) = mac.leading_comments() {
        buffer.push_str(comments);
        if !buffer.ends_with('\n') {
            buffer.push('\n');
        }
    }

    // 3) Write the macro call.
    buffer.push_str(&format!("x!{{{}}}", mac.stem()));
}

#[cfg(test)]
mod test_push_macro_with_comments {
    use super::*;

    /// Build a quick macro with or without leading comments
    fn make_macro(stem: &str, comments: Option<&str>) -> TopBlockMacro {
        TopBlockMacroBuilder::default()
            .stem(stem)
            .leading_comments(comments.map(|s| s.to_string()))
            .build()
            .unwrap()
    }

    #[traced_test]
    fn test_no_leading_comments() {
        let mut buf = String::new();
        let mac = make_macro("alpha", None);

        push_macro_with_comments(&mut buf, &mac);
        assert_eq!(buf, "x!{alpha}");
    }

    #[traced_test]
    fn test_with_leading_comments() {
        let mut buf = String::from("existing");
        let mac = make_macro("beta", Some("// doc about beta\n// more doc\n"));

        push_macro_with_comments(&mut buf, &mac);
        let expected = r#"existing
// doc about beta
// more doc
x!{beta}"#;

        assert_eq!(buf, expected);
    }

    #[traced_test]
    fn test_buffer_already_ends_with_newline() {
        let mut buf = String::from("abc\n");
        let mac = make_macro("gamma", Some("// some doc\n"));

        push_macro_with_comments(&mut buf, &mac);
        let expected = r#"abc
// some doc
x!{gamma}"#;
        assert_eq!(buf, expected);
    }

    #[traced_test]
    fn test_no_comments_on_non_empty_buf_without_newline() {
        // If the buffer doesn't end with a newline, we add one prior to the macro
        let mut buf = String::from("no_nl_at_end");
        let mac = make_macro("delta", None);

        push_macro_with_comments(&mut buf, &mac);
        let expected = "no_nl_at_end\nx!{delta}";
        assert_eq!(buf, expected);
    }
}
