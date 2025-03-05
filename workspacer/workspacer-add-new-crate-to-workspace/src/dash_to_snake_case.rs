// ---------------- [ File: src/dash_to_snake_case.rs ]
crate::ix!();

/// Helper to transform dashes => underscores for snake-case usage
pub fn dash_to_snake_case(input: &str) -> String {
    input.replace('-', "_")
}
