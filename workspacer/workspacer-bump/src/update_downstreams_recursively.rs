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

            // Get the name without holding the lock longer than needed
            let crate_name = {
                let h = arc_crate.lock().await;
                h.name().to_string()
            };

            // 2) Try to rewrite (if a `version = "…"` exists) and also detect path/workspace refs
            let (was_rewritten, references_dep) = {
                let h = arc_crate.lock().await;
                let toml = h.cargo_toml();
                let mut toml_guard = toml.lock().await;

                let rewritten =
                    toml_guard.update_dependency_version(dep_name, &new_version.to_string())?;

                // NEW: detect references even when there is no `version` key (path/workspace deps)
                let referenced =
                    toml_guard.references_dependency_in_any_table(dep_name)?;

                if rewritten {
                    toml_guard.save_to_disk().await?;
                }
                (rewritten, referenced)
            };

            // 3) Decide whether this crate needs a bump (at most once per node)
            let needs_bump = (was_rewritten || references_dep) && !visited.contains(&crate_name);
            if needs_bump {
                // Bump THIS crate's version (since its deps changed)
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

                // Recurse to crates that depend on THIS crate, carrying THIS crate's new version
                self.update_downstreams_recursively(&crate_name, &bumped_ver, release, visited).await?;
            }
        }

        Ok(())
    }
}
