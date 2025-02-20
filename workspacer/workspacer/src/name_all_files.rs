// ---------------- [ File: src/name_all_files.rs ]
crate::ix!();

// Implementation for an entire workspace. Iterates over all crates, calling `name_all_files`
// on each one. Aggregates any crate-level failures into `WorkspaceError::MultipleErrors`.
#[async_trait]
impl<P,H> NameAllFiles for Workspace<P,H>
where
    // your existing constraints:
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync,
{
    type Error = WorkspaceError;


    async fn name_all_files(&self) -> Result<(), Self::Error> {
        let mut errors = Vec::new();

        for crate_handle in self {
            if let Err(e) = crate_handle.name_all_files().await {
                // Wrap the `CrateError` in a `WorkspaceError` variant:
                errors.push(WorkspaceError::CrateError(e.into()));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(WorkspaceError::MultipleErrors(errors))
        }
    }
}
