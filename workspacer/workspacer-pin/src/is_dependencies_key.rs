// ---------------- [ File: workspacer-pin/src/is_dependencies_key.rs ]
crate::ix!();

/// Checks if a table key ends with "dependencies"
pub fn is_dependencies_key(k: &str) -> bool {
    k.ends_with("dependencies")
}
