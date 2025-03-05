// ---------------- [ File: workspacer-readme-writer/src/perform_readme_updates_for_crate.rs ]
crate::ix!();

#[async_trait]
impl PerformReadmeUpdates for CrateHandle {
    type Error = CrateError;

    #[tracing::instrument(level="trace", skip_all)]
    async fn write_updated_readme(
        &self,
        response: &AiReadmeResponse
    ) -> Result<(), Self::Error> {

        trace!("CrateHandle::write_updated_readme: about to write or update README.md");

        let readme_path = self.as_ref().join("README.md");
        debug!("Resolved readme_path: {:?}", readme_path);

        tokio::fs::write(&readme_path, response.proposed_readme_text())
            .await
            .map_err(|io_err| CrateError::IoError {
                io_error: Arc::new(io_err),
                context: format!("Failed to write README at {}", readme_path.display()),
            })?;

        info!("Successfully updated README.md at {:?}", readme_path);

        Ok(())
    }
}
