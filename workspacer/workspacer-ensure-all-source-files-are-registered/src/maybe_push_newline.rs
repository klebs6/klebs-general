crate::ix!();

/// A small helper that ensures if the `buffer` is non-empty and doesn’t
/// end with `\n`, we push exactly one newline.  
pub fn maybe_push_newline(buffer: &mut String) {
    if !buffer.is_empty() && !buffer.ends_with('\n') {
        buffer.push('\n');
    }
}

#[cfg(test)]
mod test_maybe_push_newline {
    use super::*;

    #[traced_test]
    fn test_maybe_push_newline_empty_buffer() {
        let mut buf = String::new();
        maybe_push_newline(&mut buf);
        assert!(buf.is_empty(), "Empty buffer remains empty");
    }

    #[traced_test]
    fn test_maybe_push_newline_ends_with_newline() {
        let mut buf = String::from("hello\n");
        maybe_push_newline(&mut buf);
        assert_eq!(buf, "hello\n", "Should not add another newline if already ends with \\n");
    }

    #[traced_test]
    fn test_maybe_push_newline_no_newline_at_end() {
        let mut buf = String::from("hello");
        maybe_push_newline(&mut buf);
        assert_eq!(buf, "hello\n", "Should add exactly one newline if not present");
    }

    #[traced_test]
    fn test_maybe_push_newline_already_empty_line() {
        // If buffer has something but it literally ends with an empty line?
        // It's effectively the same as 'ends_with('\n') = true'.
        let mut buf = String::from("test\n\n");
        maybe_push_newline(&mut buf);
        assert_eq!(
            buf,
            "test\n\n",
            "We do not add a newline if buffer already ends with newlines"
        );
    }
}
