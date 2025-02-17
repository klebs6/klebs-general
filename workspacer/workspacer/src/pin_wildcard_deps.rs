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
        // 1) Locate and read Cargo.lock in the root
        let lock_path = self.as_ref().join("Cargo.lock");
        if !lock_path.exists() {
            return Err(WorkspaceError::FileNotFound {
                missing_file: lock_path,
            });
        }

        info!("Reading Cargo.lock from {:?}", lock_path);

        let lockfile_str = tokio::fs::read_to_string(&lock_path).await
            .map_err(|e| WorkspaceError::IoError {
                io_error: e.into(),
                context: format!("Failed to read Cargo.lock at {:?}", lock_path),
            })?;

        // 2) Parse the lockfile
        let lockfile = cargo_lock::Lockfile::from_str(&lockfile_str)
            .map_err(|e| WorkspaceError::InvalidLockfile {
                path: lock_path.clone(),
                message: format!("{e}"),
            })?;

        // 3) Build a { crate_name -> set_of_versions } map
        use cargo_lock::Package;
        use semver::Version;
        use std::collections::{BTreeMap,BTreeSet};

        let mut lock_versions: BTreeMap<String, BTreeSet<Version>> = BTreeMap::new();
        for Package { name, version, .. } in &lockfile.packages {
            lock_versions
                .entry(name.as_str().to_owned())
                .or_default()
                .insert(version.clone());
        }

        debug!("Constructed lock_versions map with {} crates", lock_versions.len());

        // 4) Pin each crate in the workspace
        for crate_handle in self.crates() {

            let crate_path = crate_handle.as_ref().to_path_buf();

            info!("Pinning wildcard deps in crate at {:?}", crate_path);

            crate_handle
                .pin_wildcard_dependencies(&lock_versions)
                .await
                .map_err(|e| WorkspaceError::CratePinFailed {
                    crate_path,
                    source: Box::new(e),
                })?;
        }

        Ok(())
    }
}
