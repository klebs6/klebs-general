// ---------------- [ File: workspacer-workspace/src/validate_integrity.rs ]
crate::ix!();

impl<P, H> ValidateIntegrity for Workspace<P, H>
where
    H: CrateHandleInterface<P>,
    // We still need P: ... 'async_trait, but we specifically DO NOT require `'static` for H.
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    type Error = WorkspaceError;

    fn validate_integrity(&self) -> Result<(), Self::Error> {

        trace!("Starting Workspace::validate_integrity without forcing 'static on H");

        // Copy all Arc<AsyncMutex<H>> items out of &self into a local Vec.
        // That way, the async block does NOT borrow from self at all.
        let crates_to_validate: Vec<_> = self.crates().iter().cloned().collect();

        // Instead of using run_async_without_nested_runtime (which typically
        // requires its future to be `'static`), we can do a one-shot `block_on`
        // from the `futures` crate. That block_on does NOT require `'static`.
        // We just need to pin our async block on the stack:
        let fut = async move {
            for crate_arc in crates_to_validate {
                trace!("Locking one crate_arc in async block to validate integrity");
                let guard = crate_arc.lock().await;
                guard.validate_integrity()?;
            }
            Ok::<(), Self::Error>(())
        };

        // Pin the future on the stack so it doesn't need to be `'static`.
        futures::pin_mut!(fut);

        // Now we can synchronously wait on `fut` with the `futures` executor
        // — which does *not* impose the `'static` bound on the future.
        match futures::executor::block_on(fut) {
            Ok(()) => {
                debug!("All crates validated successfully.");
                Ok(())
            }
            Err(e) => {
                error!("Validation error: {:?}", e);
                Err(e)
            }
        }
    }
}
