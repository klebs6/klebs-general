// ---------------- [ File: src/maybe_ensure_newline.rs ]
crate::ix!();

/// If there is content in `buffer` and it does not end with a newline,
/// push exactly one `\n`.
pub fn maybe_ensure_newline(buffer: &mut String) {
    if !buffer.is_empty() && !buffer.ends_with('\n') {
        buffer.push('\n');
    }
}
