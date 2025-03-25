// ---------------- [ File: workspacer-pin/src/pin_wildcard_deps.rs ]
crate::ix!();

#[async_trait]
pub trait PinAllWildcardDependencies {
    type Error;
    async fn pin_all_wildcard_dependencies(&mut self) -> Result<(), Self::Error>;
}

// (B) Ensure that for the Workspace code, we require that `H: PinWildcardDependencies<Error=CrateError>`
//     so that we can call `guard.pin_wildcard_dependencies(...)` on `MutexGuard<H>`.
#[async_trait]
impl<P, H> PinAllWildcardDependencies for Workspace<P, H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    for<'async_trait> H: CrateHandleInterface<P>
        + PinWildcardDependencies<Error = CrateError>
        + Send
        + Sync
        + 'async_trait,
{
    type Error = WorkspaceError;

    async fn pin_all_wildcard_dependencies(&mut self) -> Result<(), Self::Error> {
        info!("pin_all_wildcard_dependencies for Workspace at {:?}", self.as_ref());

        let lock_versions = match build_lock_versions(self).await {
            Ok(lv) => lv,
            Err(crate_err) => {
                return Err(WorkspaceError::CratePinFailed {
                    crate_path: self.as_ref().to_path_buf(),
                    source: Box::new(crate_err),
                });
            }
        };

        for arc_crate in self.crates() {
            let mut guard = arc_crate.lock().expect("expected to lock crate");
            debug!("pin_all_wildcard_dependencies: pinning a crate in the workspace...");
            guard
                .pin_wildcard_dependencies(&lock_versions)
                .await
                .map_err(|crate_err| WorkspaceError::CratePinFailed {
                    crate_path: guard.as_ref().to_path_buf(),
                    source: Box::new(crate_err),
                })?;
        }

        Ok(())
    }
}
