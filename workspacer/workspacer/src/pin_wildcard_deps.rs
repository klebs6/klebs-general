// ---------------- [ File: workspacer/src/pin_wildcard_deps.rs ]
crate::ix!();

#[async_trait]
impl<P,H> PinAllWildcardDependencies for Workspace<P,H>
where
    // your existing constraints:
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync,
{
    type Error = WorkspaceError;

    /// Loads `Cargo.lock` from the workspace root, collects crate versions,
    /// then iterates over each crate and calls `pin_wildcard_dependencies`.
    async fn pin_all_wildcard_dependencies(&self) -> Result<(), Self::Error> {
        info!("pin_all_wildcard_dependencies called for Workspace at {:?}", self.as_ref());
        // 1) Build lock_versions from workspace root
        let lock_versions = match workspacer_toml::build_lock_versions(self).await {
            Ok(map) => map,
            Err(e) => {
                // Convert `CrateError` to `WorkspaceError`
                return match e {
                    CrateError::FileNotFound { missing_file } => {
                        Err(WorkspaceError::FileNotFound { missing_file })
                    }
                    CrateError::LockfileParseFailed { path, message } => {
                        Err(WorkspaceError::InvalidLockfile { path, message })
                    }
                    CrateError::IoError { io_error, context } => {
                        Err(WorkspaceError::IoError { io_error, context })
                    }
                    other => {
                        // any others you want to map
                        Err(WorkspaceError::InvalidWorkspace {
                            invalid_workspace_path: self.as_ref().to_path_buf()
                        })
                    }
                };
            }
        };

        // 2) For each crate in the workspace, pin
        for c in self.crates() {
            info!("Pinning wildcard deps in crate at {:?}", c.as_ref());
            c.pin_wildcard_dependencies(&lock_versions)
                .await
                .map_err(|crate_err| {
                    // map CrateError -> WorkspaceError
                    WorkspaceError::CratePinFailed {
                        crate_path: c.as_ref().to_path_buf(),
                        source: Box::new(crate_err),
                    }
                })?;
        }
        Ok(())
    }
}
