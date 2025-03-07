// ---------------- [ File: token-expander-axis-derive/src/strip_surrounding_quotes.rs ]
crate::ix!();

/// Naively strips leading/trailing quotes if present. 
/// For more safety, parse as a `LitStr` with `syn`.
pub fn strip_surrounding_quotes(s: &str) -> String {
    let mut text = s.trim().to_string();
    if text.starts_with('\"') && text.ends_with('\"') && text.len() >= 2 {
        text.remove(0);
        text.pop();
    }
    text
    // In production, you may want to handle escaping, etc.
}
