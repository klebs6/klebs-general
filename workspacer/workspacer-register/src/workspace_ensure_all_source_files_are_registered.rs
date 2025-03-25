// ---------------- [ File: workspacer-register/src/workspace_ensure_all_source_files_are_registered.rs ]
crate::ix!();

#[async_trait]
impl<P,H> EnsureAllSourceFilesAreRegistered for Workspace<P,H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + EnsureAllSourceFilesAreRegistered<Error=SourceFileRegistrationError> + Send + Sync,
{
    type Error = SourceFileRegistrationError;

    async fn ensure_all_source_files_are_registered(&self) -> Result<(), Self::Error> {
        trace!("Entering Workspace<P,H>::ensure_all_source_files_are_registered");

        for crate_handle in self.crates() {
            let guard = crate_handle.lock().await;
            debug!("Ensuring source files registered for crate '{}'", guard.name());
            guard.ensure_all_source_files_are_registered().await?;
        }

        trace!("Exiting Workspace<P,H>::ensure_all_source_files_are_registered");
        Ok(())
    }
}
