// ---------------- [ File: src/strip_leading_dot_slash.rs ]
crate::ix!();

/// Helper to remove leading "./" from a &str
pub fn strip_leading_dot_slash(s: &str) -> &str {
    if let Some(stripped) = s.strip_prefix("./") {
        stripped
    } else {
        s
    }
}
