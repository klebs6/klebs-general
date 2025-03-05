// ---------------- [ File: workspacer-readme-writer/src/perform_readme_updates_for_workspace.rs ]
crate::ix!();

#[async_trait]
impl<P,H> PerformReadmeUpdates for Workspace<P,H>
where
    // Again, match the same bounds required by `Workspace`:
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    for<'async_trait> H: CrateHandleInterface<P>
        + GenerateReadmeQueries<Error = CrateError>
        + PerformReadmeUpdates<Error = CrateError>
        + ConsolidateCrateInterface
        + Sync
        + Send
        + 'async_trait,
{
    type Error = WorkspaceError;

    #[tracing::instrument(level="trace", skip_all)]
    async fn write_updated_readme(
        &self,
        response: &AiReadmeResponse
    ) -> Result<(), Self::Error> {
        trace!("Workspace::write_updated_readme: applying top-level workspace readme update (if any).");

        let readme_path = self.as_ref().join("README.md");
        debug!("Resolved workspace-level readme_path: {:?}", readme_path);

        tokio::fs::write(&readme_path, response.proposed_readme_text())
            .await
            .map_err(|io_err| WorkspaceError::IoError {
                io_error: Arc::new(io_err),
                context: format!("Failed to write top-level workspace README at {}", readme_path.display()),
            })?;

        info!("Successfully updated top-level README.md for workspace at {:?}", readme_path);
        Ok(())
    }
}
