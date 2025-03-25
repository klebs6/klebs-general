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
            let handle = crate_handle.clone();
            run_async_without_nested_runtime(async move {
                trace!("Locking cargo_toml_handle in async block to validate CargoToml");
                let guard = handle.lock().await;
                guard.validate_integrity()?;
                Ok::<(),WorkspaceError>(())
            })?;
        }
        Ok(())
    }
}
