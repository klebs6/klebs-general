// ---------------- [ File: workspacer-workspace/src/validate_integrity.rs ]
crate::ix!();

#[async_trait]
impl<P, H> ValidateIntegrity for Workspace<P, H>
where
    H: CrateHandleInterface<P>,
    // We still need P: ... 'async_trait, but we specifically DO NOT require `'static` for H.
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    type Error = WorkspaceError;

    async fn validate_integrity(&self) -> Result<(), Self::Error> {
        trace!("Starting Workspace::validate_integrity without forcing 'static on H");
        for crate_arc in self.crates() {
            trace!("Locking one crate_arc in async block to validate integrity");
            let guard = crate_arc.lock().await;
            guard.validate_integrity().await?;
        }
        Ok(())
    }
}
