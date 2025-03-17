// ---------------- [ File: workspacer-bump/src/update_downstreams_recursively.rs ]
crate::ix!();

#[async_trait]
pub trait WorkspaceDownstreamExt {
    async fn update_downstreams_recursively(
        &mut self,
        dep_name: &str,
        new_version: &semver::Version,
        visited: &mut HashSet<String>,
    ) -> Result<(), WorkspaceError>;
}

#[async_trait]
impl<P, H> WorkspaceDownstreamExt for Workspace<P, H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Bump<Error = CrateError> + Send + Sync,
{
    async fn update_downstreams_recursively(
        &mut self,
        dep_name: &str,
        new_version: &semver::Version,
        visited: &mut HashSet<String>,
    ) -> Result<(), WorkspaceError> {
        use tracing::{trace, debug, warn};

        let crates_list: Vec<_> = self.crates().iter().cloned().collect();
        // Now we’re not borrowing `self` immutably any more.

        for arc_crate in crates_list {
            // Lock once to figure out crate_name
            let crate_name = {
                let handle = arc_crate.lock().unwrap();
                handle.name().to_string()
            };

            if visited.contains(&crate_name) {
                continue;
            }

            // Next, lock again for the update:
            let mut handle = arc_crate.lock().unwrap();
            let cargo_toml_arc = handle.cargo_toml();
            let mut cargo_toml_guard = cargo_toml_arc.lock().unwrap();

            let changed = cargo_toml_guard.update_dependency_version(dep_name, &new_version.to_string())?;

            drop(cargo_toml_guard); // <— drop guard before `.await`

            if changed {
                // Save new version => must reacquire or have a separate method
                // or do it in the same guard scope if your saving is synchronous.  
                // If it’s truly async, do the pattern: copy out data => drop guard => do .await
                let mut cargo_toml_guard = arc_crate
                    .lock()
                    .unwrap()
                    .cargo_toml()
                    .lock()
                    .unwrap();
                cargo_toml_guard.save_to_disk().await?;

                visited.insert(crate_name.clone());

                // Recurse:
                self.update_downstreams_recursively(&crate_name, new_version, visited).await?;
            }
        }

        Ok(())
    }

}
