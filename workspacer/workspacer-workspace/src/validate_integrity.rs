// ---------------- [ File: workspacer-workspace/src/validate_integrity.rs ]
crate::ix!();

impl<P,H:CrateHandleInterface<P>> ValidateIntegrity for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    /// Validates the integrity of the entire workspace by checking each crate
    ///
    fn validate_integrity(&self) -> Result<(), Self::Error> {
        for crate_handle in self {
            crate_handle.lock().expect("expected to be able to lock our crate").validate_integrity()?;
        }
        Ok(())
    }
}
