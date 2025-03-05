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
impl<P,H> WorkspaceDownstreamExt for Workspace<P,H>
where
    P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static,
    H: CrateHandleInterface<P> + Bump<Error=CrateError> + Send + Sync,
{
    /// Recursively updates any crates that depend on `dep_name` to use `new_version`.
    /// We convert any inline table references to standard tables for consistency,
    /// and do a best-effort approach (no global failure if one parse fails, as the tests want).
    async fn update_downstreams_recursively(
        &mut self,
        dep_name: &str,
        new_version: &semver::Version,
        visited: &mut HashSet<String>,
    ) -> Result<(), WorkspaceError> {
        trace!("Updating downstreams for dep='{}' => new_version={}", dep_name, new_version);
        let new_version_str = new_version.to_string();

        // gather crate names first
        let crate_names: Vec<String> = self.crates().iter().map(|ch| ch.name().to_string()).collect();

        for c_name in crate_names {
            if visited.contains(&c_name) {
                continue;
            }

            let maybe_handle = self.crates_mut().iter_mut().find(|ch| ch.name() == c_name);
            if maybe_handle.is_none() {
                continue;
            }
            let crate_handle = maybe_handle.unwrap();

            let cargo_toml_path = crate_handle.as_ref().join("Cargo.toml");

            // parse
            let contents = match fs::read_to_string(&cargo_toml_path).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("Cannot read {:?} while updating downstreams, skipping: {}", cargo_toml_path, e);
                    continue;
                }
            };
            let mut doc = match contents.parse::<TomlEditDocument>() {
                Ok(d) => d,
                Err(e) => {
                    warn!("Parse error on {:?} while updating downstreams, skipping: {}", cargo_toml_path, e);
                    continue;
                }
            };

            // attempt to update references
            let mut found_dep = false;
            for deps_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(deps_tbl) = doc.get_mut(deps_key).and_then(|i| i.as_table_mut()) {
                    if let Some(dep_item) = deps_tbl.get_mut(dep_name) {
                        // convert inline => normal table if needed
                        if let Some(inline) = dep_item.as_inline_table_mut() {
                            trace!("Converting inline table in {} -> dep='{}'", c_name, dep_name);
                            let as_table = inline.clone().into_table();
                            *dep_item = TomlEditItem::Table(as_table);
                        }

                        if dep_item.is_table() {
                            trace!("Updating table reference in crate='{}' for dep='{}' => '{}'",
                                   c_name, dep_name, new_version_str);
                            let t = dep_item.as_table_mut().unwrap();
                            t.insert("version", TomlEditItem::Value(TomlEditValue::from(new_version_str.clone())));
                            found_dep = true;
                        } else if dep_item.is_str() {
                            trace!("Updating string reference in crate='{}' for dep='{}' => '{}'",
                                   c_name, dep_name, new_version_str);
                            *dep_item = TomlEditItem::Value(TomlEditValue::from(new_version_str.clone()));
                            found_dep = true;
                        } else {
                            debug!("Dependency '{}' in crate='{}' not a table or string, skipping", dep_name, c_name);
                        }
                    }
                }
            }

            if found_dep {
                debug!("Rewriting Cargo.toml for crate='{}' => path={:?}", c_name, cargo_toml_path);
                if let Err(e) = fs::write(&cargo_toml_path, doc.to_string()).await {
                    warn!("Failed rewriting references in {:?}: {}", cargo_toml_path, e);
                }
                visited.insert(c_name.clone());

                // Recurse further
                self.update_downstreams_recursively(&c_name, new_version, visited).await?;
            }
        }
        Ok(())
    }
}
