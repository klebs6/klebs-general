// ---------------- [ File: workspacer-readme-writer/src/perform_readme_updates.rs ]
crate::ix!();

/// A trait for performing the actual creation or update of the README after
/// we obtain the AI’s proposed README text. This trait does the final write
/// to disk or sets up the new file content as needed.
#[async_trait]
pub trait PerformReadmeUpdates {
    type Error;

    /// Takes the final AI response (the proposed README text, plus commentary if any)
    /// and either creates a new README.md or updates the existing one accordingly.
    async fn write_updated_readme(
        &self,
        response: &AiReadmeResponse
    ) -> Result<(), Self::Error>;
}
