crate::ix!();

/// Custom error type for handling name-related issues.
#[derive(Debug)]
pub enum NameError {
    InvalidName(String),
    DuplicateName(String),
    EmptyName,
}
