// ---------------- [ File: workspacer-bump/src/update_downstreams_recursively.rs ]
crate::ix!();

#[async_trait]
pub trait WorkspaceDownstreamExt {
    async fn update_downstreams_recursively(
        &mut self,
        dep_name:    &str,
        new_version: &semver::Version,
        release:     &ReleaseType,
        visited:     &mut HashSet<String>,
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
        dep_name:    &str,
        new_version: &semver::Version,
        release:     &ReleaseType,
        visited:     &mut HashSet<String>,
    ) -> Result<(), WorkspaceError> {

        // 1) local copy of crates
        let crate_list: Vec<_> = self.crates().iter().cloned().collect();

        for arc_crate in crate_list {
            let crate_name = {
                let h = arc_crate.lock().await;
                h.name().to_string()
            };
            if visited.contains(&crate_name) {
                continue;
            }

            // 2) lock for short, synchronous update
            let changed = {

                let h          = arc_crate.lock().await;
                let toml           = h.cargo_toml();
                let mut toml_guard = toml.lock().await;

                // do in-memory updates
                let changed = toml_guard.update_dependency_version(dep_name, &new_version.to_string())?;

                changed
                // guard dropped
            };

            if changed {
                // 1) Save the dependency rewrite
                {
                    let crate_guard = arc_crate.lock().await;
                    let toml = crate_guard.cargo_toml();
                    let toml_guard = toml.lock().await;
                    toml_guard.save_to_disk().await?;
                }

                // 2) Bump THIS crate's version (since its deps changed)
                let bumped_ver = {
                    let mut guard = arc_crate.lock().await;
                    guard.bump(release.clone()).await.map_err(|e| {
                        WorkspaceError::BumpError {
                            crate_path: guard.as_ref().join("Cargo.toml"),
                            source: Box::new(e),
                        }
                    })?;
                    guard.version().map_err(WorkspaceError::CrateError)?
                };

                visited.insert(crate_name.clone());

                // 3) Recurse to crates that depend on THIS crate, carrying THIS crate's new version
                self.update_downstreams_recursively(&crate_name, &bumped_ver, release, visited).await?;
            }

        }

        Ok(())
    }
}
