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
    // No 'static on H
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Bump<Error=CrateError> + Send + Sync,
{
    async fn update_downstreams_recursively(
        &mut self,
        dep_name: &str,
        new_version: &semver::Version,
        visited: &mut HashSet<String>,
    ) -> Result<(), WorkspaceError> {
        trace!("Updating downstreams for dep='{}' => new_ver={}", dep_name, new_version);
        let new_version_str = new_version.to_string();

        let crate_names = self.get_all_crate_names();
        for c_name in crate_names {
            if visited.contains(&c_name) {
                continue;
            }
            let arc_crate = match self.find_crate_by_name(&c_name) {
                Some(a) => a,
                None => continue,
            };
            let crate_path = {
                let g = arc_crate.lock().expect("mutex lock for path");
                g.as_ref().to_path_buf()
            };
            let cargo_toml_path = crate_path.join("Cargo.toml");
            let contents = match fs::read_to_string(&cargo_toml_path).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("Cannot read {:?}: {}, skipping", cargo_toml_path, e);
                    continue;
                }
            };
            let mut doc = match contents.parse::<toml_edit::Document>() {
                Ok(d) => d,
                Err(e) => {
                    warn!("Parse error on {:?}: {}, skipping", cargo_toml_path, e);
                    continue;
                }
            };

            let mut found = false;
            for key in ["dependencies","dev-dependencies","build-dependencies"] {
                if let Some(tbl) = doc.get_mut(key).and_then(|x| x.as_table_mut()) {
                    if let Some(dep_item) = tbl.get_mut(dep_name) {
                        if let Some(inline) = dep_item.as_inline_table_mut() {
                            let expanded = inline.clone().into_table();
                            *dep_item = toml_edit::Item::Table(expanded);
                        }
                        if dep_item.is_table() {
                            dep_item
                                .as_table_mut()
                                .unwrap()
                                .insert("version", toml_edit::value(new_version_str.clone()));
                            found = true;
                        } else if dep_item.is_str() {
                            *dep_item = toml_edit::value(new_version_str.clone());
                            found = true;
                        }
                    }
                }
            }
            if found {
                debug!("Rewriting references in {:?}", cargo_toml_path);
                if let Err(e) = fs::write(&cargo_toml_path, doc.to_string()).await {
                    warn!("Cannot rewrite references in {:?}: {}", cargo_toml_path, e);
                }
                visited.insert(c_name.clone());
                self.update_downstreams_recursively(&c_name, new_version, visited).await?;
            }
        }

        Ok(())
    }
}
