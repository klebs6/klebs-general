// ---------------- [ File: src/expected_content_type.rs ]
crate::ix!();

#[derive(Debug, Clone, Copy)]
pub enum ExpectedContentType {
    /// We’re expecting JSON in the response—always attempt to parse/repair.
    Json,
    /// We’re expecting just raw text—no JSON parsing at all.
    PlainText,
    /// (ADDED) Some tests refer to “JsonLines”. We add it here so they compile.
    JsonLines,
}
