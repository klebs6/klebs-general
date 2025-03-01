// ---------------- [ File: src/sort_and_format_imports_for_workspace.rs ]
crate::ix!();

#[async_trait]
impl<P,H> SortAndFormatImports for Workspace<P,H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<std::path::Path> + Clone + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + SortAndFormatImports<Error=CrateError> + Send + Sync,
{
    type Error = WorkspaceError;

    async fn sort_and_format_imports(&self) -> Result<(), Self::Error> {
        for crate_handle in self.into_iter() {
            debug!("Sorting/formatting imports for crate: {}", crate_handle.name());
            crate_handle.sort_and_format_imports().await
                .map_err(|ce| WorkspaceError::CrateError(ce))?;
        }
        Ok(())
    }
}
