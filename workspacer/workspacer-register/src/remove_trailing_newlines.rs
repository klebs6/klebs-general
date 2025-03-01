// ---------------- [ File: src/remove_trailing_newlines.rs ]
crate::ix!();

/// Removes trailing newlines from the provided buffer string (in-place).
pub fn remove_trailing_newlines(buffer: &mut String) {
    trace!("Entering remove_trailing_newlines");
    while buffer.ends_with('\n') {
        buffer.pop();
    }
    debug!("remove_trailing_newlines => final length={}", buffer.len());
    trace!("Exiting remove_trailing_newlines");
}

#[cfg(test)]
mod test_remove_trailing_newlines {
    use super::*;

    #[traced_test]
    fn test_no_trailing_newline() {
        let mut buf = String::from("hello");
        remove_trailing_newlines(&mut buf);
        assert_eq!(buf, "hello");
    }

    #[traced_test]
    fn test_single_trailing_newline() {
        let mut buf = String::from("hello\n");
        remove_trailing_newlines(&mut buf);
        assert_eq!(buf, "hello");
    }

    #[traced_test]
    fn test_multiple_trailing_newlines() {
        let mut buf = String::from("line\n\n\n");
        remove_trailing_newlines(&mut buf);
        assert_eq!(buf, "line");
    }

    #[traced_test]
    fn test_empty_buffer() {
        let mut buf = String::new();
        remove_trailing_newlines(&mut buf);
        assert!(buf.is_empty(), "Still empty");
    }

    #[traced_test]
    fn test_only_newlines() {
        let mut buf = String::from("\n\n\n");
        remove_trailing_newlines(&mut buf);
        assert!(buf.is_empty(), "All newlines removed => empty");
    }
}
