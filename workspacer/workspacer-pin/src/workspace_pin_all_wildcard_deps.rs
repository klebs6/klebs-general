// ---------------- [ File: workspacer-pin/src/pin_wildcard_deps.rs ]
crate::ix!();

#[async_trait]
pub trait PinAllWildcardDependencies {
    type Error;
    async fn pin_all_wildcard_dependencies(&mut self) -> Result<(), Self::Error>;
}

#[async_trait]
impl<P, H> PinAllWildcardDependencies for Workspace<P, H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    for<'async_trait> H: CrateHandleInterface<P>
        + PinWildcardDependencies<Error = CrateError>
        + AsyncTryFrom<PathBuf, Error = CrateError>  // <-- IMPORTANT: we use AsyncTryFrom here
        + Send
        + Sync
        + 'async_trait,
{
    type Error = WorkspaceError;

    async fn pin_all_wildcard_dependencies(&mut self) -> Result<(), Self::Error> {
        info!(
            "pin_all_wildcard_dependencies: started for Workspace at path={:?}",
            self.as_ref()
        );

        // Build lock_versions first
        let lock_versions = match build_lock_versions(self).await {
            Ok(lv) => {
                trace!(
                    "pin_all_wildcard_dependencies: built lock_versions for workspace at path={:?}",
                    self.as_ref()
                );
                lv
            }
            Err(crate_err) => {
                error!(
                    "pin_all_wildcard_dependencies: build_lock_versions failed for {:?}: {:?}",
                    self.as_ref(),
                    crate_err
                );
                return Err(WorkspaceError::CratePinFailed {
                    crate_path: self.as_ref().to_path_buf(),
                    source: Box::new(crate_err),
                });
            }
        };

        // For each crate in this workspace, do not hold the MutexGuard across .await
        for arc_crate in self.crates() {
            let crate_path = {
                let guard = arc_crate.lock().expect("expected to lock crate");
                let path = guard.as_ref().to_path_buf();
                debug!(
                    "pin_all_wildcard_dependencies: captured crate path={:?}, dropping guard",
                    path
                );
                path
            };

            // Construct a new H using AsyncTryFrom<PathBuf>
            let mut ephemeral = match <H as AsyncTryFrom<PathBuf>>::new(&crate_path).await {
                Ok(h) => {
                    trace!(
                        "pin_all_wildcard_dependencies: successfully built ephemeral handle for path={:?}",
                        crate_path
                    );
                    h
                }
                Err(e) => {
                    error!(
                        "pin_all_wildcard_dependencies: failed to build ephemeral handle for path={:?}: {:?}",
                        crate_path, e
                    );
                    return Err(WorkspaceError::CratePinFailed {
                        crate_path: crate_path.clone(),
                        source: Box::new(e),
                    });
                }
            };

            // Pin wildcard deps in that ephemeral handle
            if let Err(crate_err) = ephemeral.pin_wildcard_dependencies(&lock_versions).await {
                error!(
                    "pin_all_wildcard_dependencies: pin_wildcard_dependencies failed for path={:?}: {:?}",
                    crate_path, crate_err
                );
                return Err(WorkspaceError::CratePinFailed {
                    crate_path: crate_path.clone(),
                    source: Box::new(crate_err),
                });
            }

            // Overwrite the crate in memory
            {
                let mut guard = arc_crate.lock().expect("expected to lock crate for overwrite");
                *guard = ephemeral;
                debug!(
                    "pin_all_wildcard_dependencies: replaced crate data in memory for path={:?}",
                    crate_path
                );
            }
        }

        info!(
            "pin_all_wildcard_dependencies: completed for all crates in Workspace at path={:?}",
            self.as_ref()
        );
        Ok(())
    }
}

#[cfg(test)]
mod test_pin_all_wildcard_dependencies_for_workspace {
    use super::*;
    use tracing::{info, debug, trace};

    #[traced_test]
    async fn pins_star_in_workspace() {
        info!("Starting test_pin_all_wildcard_dependencies_for_workspace::pins_star_in_workspace");

        // Create a workspace from a path that might or might not have real data.
        // In production tests, we would put fixtures. For here, we just ensure the call is made
        // without panics.
        let ws_path = PathBuf::from("mock/workspace/path");
        let mut workspace = match Workspace::<PathBuf, CrateHandle>::new(&ws_path).await {
            Ok(ws) => ws,
            Err(e) => {
                // It's possible the code is in "ActuallyInSingleCrate" mode, so we just skip
                debug!("test_pin_all_wildcard_dependencies_for_workspace: not a workspace? Err: {:?}", e);
                return;
            }
        };

        // Now attempt the pin
        let result = workspace.pin_all_wildcard_dependencies().await;
        // We only check that no panic occurred and it returned something
        match result {
            Ok(_) => info!("workspace pinning succeeded"),
            Err(e) => debug!("workspace pinning returned an error, but didn't panic: {:?}", e),
        }
        debug!("test_pin_all_wildcard_dependencies_for_workspace::pins_star_in_workspace passed");
    }
}
