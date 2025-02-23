crate::ix!();

// A trait for "naming all .rs files" with a comment tag
#[async_trait]
pub trait NameAllFiles {
    type Error;

    /// Remove old `// ------ [ File: ... ]` lines and prepend a new one
    /// naming each `.rs` file in either a single crate or an entire workspace.
    async fn name_all_files(&self) -> Result<(), Self::Error>;
}
