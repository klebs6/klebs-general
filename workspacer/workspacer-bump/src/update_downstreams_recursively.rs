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

        // 1) local copy of crates
        let crate_list: Vec<_> = self.crates().iter().cloned().collect();

        for arc_crate in crate_list {
            let crate_name = {
                let h = arc_crate.lock().unwrap();
                h.name().to_string()
            };
            if visited.contains(&crate_name) {
                continue;
            }

            // 2) lock for short, synchronous update
            let changed = {

                let mut h          = arc_crate.lock().unwrap();
                let toml           = h.cargo_toml();
                let mut toml_guard = toml.lock().unwrap();

                // do in-memory updates
                let changed = toml_guard.update_dependency_version(dep_name, &new_version.to_string())?;

                changed
                // guard dropped
            };

            if changed {
                // 3) do async save, once the guard is dropped
                {
                    let mut toml = arc_crate
                        .lock().unwrap()
                        .cargo_toml()
                        .lock().unwrap();
                    toml.save_to_disk().await?;
                }

                visited.insert(crate_name.clone());
                self.update_downstreams_recursively(&crate_name, new_version, visited).await?;
            }
        }

        Ok(())
    }
}
