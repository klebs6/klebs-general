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
        + AsyncTryFrom<PathBuf, Error = CrateError>
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

        // --------------------------------------------------------------------
        // 0) Build `lock_versions` ONCE – this is reused for the workspace root
        //    and for every crate we iterate over later.
        // --------------------------------------------------------------------
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

        // --------------------------------------------------------------------
        // 1) **NEW** – pin wildcard dependencies in the *top‑level* Cargo.toml.
        // --------------------------------------------------------------------
        {
            let root_cargo_toml_path = self.as_ref().join("Cargo.toml");
            trace!(
                "pin_all_wildcard_dependencies: pinning wildcard deps in workspace root Cargo.toml at {:?}",
                root_cargo_toml_path
            );

            // Load an *ephemeral* handle so we never hold a lock across await
            let mut root_ctoml = workspacer_toml::CargoToml::new(&root_cargo_toml_path)
                .await
                .map_err(WorkspaceError::InvalidCargoToml)?;

            root_ctoml
                .pin_wildcard_dependencies(&lock_versions)
                .await
                .map_err(WorkspaceError::InvalidCargoToml)?;

            info!(
                "pin_all_wildcard_dependencies: successfully pinned wildcard deps in workspace root Cargo.toml at {:?}",
                root_cargo_toml_path
            );
        }

        // --------------------------------------------------------------------
        // 2) Iterate over every crate in the workspace (unchanged logic apart
        //    from updated logging strings to reflect the new two‑step process).
        // --------------------------------------------------------------------
        for arc_crate in self.crates() {
            // a) Capture the path **without** holding the guard across `.await`
            let crate_path = {
                let guard = arc_crate.lock().await;
                let path = guard.as_ref().to_path_buf();
                debug!(
                    "pin_all_wildcard_dependencies: captured crate path={:?}, dropping guard",
                    path
                );
                path
            };

            // b) Re‑instantiate a fresh handle so mutations are isolated
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

            // c) Pin the crate’s wildcard deps
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

            // d) Replace the in‑memory crate with the newly‑pinned one
            {
                let mut guard = arc_crate.lock().await;
                *guard = ephemeral;
                debug!(
                    "pin_all_wildcard_dependencies: replaced crate data in memory for path={:?}",
                    crate_path
                );
            }
        }

        info!(
            "pin_all_wildcard_dependencies: completed for workspace root and all member crates at path={:?}",
            self.as_ref()
        );
        Ok(())
    }
}
