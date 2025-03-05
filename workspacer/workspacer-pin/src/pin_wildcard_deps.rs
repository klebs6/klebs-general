// ---------------- [ File: workspacer-pin/src/pin_wildcard_deps.rs ]
crate::ix!();

#[async_trait]
pub trait PinAllWildcardDependencies {
    type Error;
    async fn pin_all_wildcard_dependencies(&self) -> Result<(), Self::Error>;
}

#[async_trait]
impl<P,H> PinAllWildcardDependencies for Workspace<P,H>
where
    // your existing constraints:
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    H: PinWildcardDependencies<Error=CrateError> + CrateHandleInterface<P> + Send + Sync,
{
    type Error = WorkspaceError;

    /// Loads `Cargo.lock` from the workspace root, collects crate versions,
    /// then iterates over each crate and calls `pin_wildcard_dependencies`.
    async fn pin_all_wildcard_dependencies(&self) -> Result<(), Self::Error> {
        info!("pin_all_wildcard_dependencies called for Workspace at {:?}", self.as_ref());
        // 1) Build lock_versions from workspace root
        let lock_versions = match build_lock_versions(self).await {
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

#[cfg(test)]
#[disable]
mod test_pin_all_wildcard_dependencies_real {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use workspacer_3p::tokio;
    // Import your real workspace type, e.g. `Workspace<P,H>`, plus the trait:
    //   use crate::{ PinAllWildcardDependencies, etc. };

    #[tokio::test]
    async fn test_pin_all_wildcard_dependencies_succeeds() {
        // 1) Create a temporary workspace with multiple crates that have wildcard deps.
        // For instance, crate_a's Cargo.toml might have something like:
        //   version = "*"
        // or a dependency on crate_b = "*", etc.  
        // We'll assume you have a helper like create_mock_workspace that can do this.
        let root_path = create_mock_workspace_with_wildcards().await
            .expect("Failed to create mock workspace with wildcard deps");

        // Possibly run `cargo update` or something to generate a Cargo.lock, or manually create one.
        // Ensure the lockfile is present and valid.

        // 2) Build the real workspace
        let ws = Workspace::new(&root_path)
            .await
            .expect("Should create workspace from mock dir");

        // 3) Call pin_all_wildcard_dependencies
        let result = ws.pin_all_wildcard_dependencies().await;

        // 4) Expect success if lockfile was valid and crates pinned successfully
        assert!(result.is_ok(), "Expected successful pinning with valid lockfile and no crate errors");

        // Optionally, you can verify each crate's Cargo.toml is updated to a pinned version
        // if that’s what `pin_wildcard_dependencies` does. For example, read the new Cargo.toml
        // or check that the wildcard is replaced with a semver from the lockfile.
        // ...
    }

    /// If there's no Cargo.lock or it's invalid, we expect an error mapping to `WorkspaceError`.
    #[tokio::test]
    async fn test_pin_all_wildcard_dependencies_lockfile_missing() {
        let root_path = create_workspace_without_lockfile().await
            .expect("some mock creation");

        let ws = Workspace::new(&root_path).await.unwrap();

        let result = ws.pin_all_wildcard_dependencies().await;
        match result {
            Err(WorkspaceError::FileNotFound { .. }) => {
                // Good: no Cargo.lock => we get that error
            }
            other => panic!("Expected FileNotFound for missing Cargo.lock, got {:?}", other),
        }
    }

    /// If a single crate fails to pin (e.g. parse error in Cargo.toml), we expect
    /// `WorkspaceError::CratePinFailed { crate_path, source }`.
    #[tokio::test]
    async fn test_pin_all_wildcard_dependencies_one_crate_fails() {
        let root_path = create_mock_workspace_with_one_broken_crate().await
            .expect("mock with one broken crate");

        let ws = Workspace::new(&root_path).await.unwrap();
        let result = ws.pin_all_wildcard_dependencies().await;

        match result {
            Err(WorkspaceError::CratePinFailed { crate_path, source }) => {
                // This is the error we expect if exactly one crate fails
                println!("Pin failed in crate {:?} with error: {:?}", crate_path, source);
            }
            Ok(_) => panic!("Expected one crate to fail => should have returned CratePinFailed"),
            other => panic!("Expected CratePinFailed, got {:?}", other),
        }
    }
}
