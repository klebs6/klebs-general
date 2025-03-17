// ---------------- [ File: workspacer-bump/src/workspace_bump_all.rs ]
crate::ix!();

#[async_trait]
impl<P, H> BumpAll for Workspace<P, H>
where
    // P must allow us to do From<PathBuf> + AsRef<Path>, etc.
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    for<'async_trait> H: CrateHandleInterface<P>
        + Bump<Error = CrateError>
        + AsyncTryFrom<PathBuf, Error = CrateError>
        + Send
        + Sync
        + 'async_trait,
{
    type Error = WorkspaceError;

    /// Bump all crates in the workspace by the specified release type.
    #[tracing::instrument(level="trace", skip_all, fields(release=?release))]
    async fn bump_all(&mut self, release: ReleaseType) -> Result<(), Self::Error> {
        trace!("Starting bump_all with release={:?}", release);

        // -------------------------------------------------------------
        // 1) Validate that the top-level Cargo.toml has a [workspace]
        // -------------------------------------------------------------
        let top_level_cargo = self.as_ref().join("Cargo.toml");
        let top_level_str = fs::read_to_string(&top_level_cargo).await.map_err(|io_err| {
            error!("Unable to read top-level Cargo.toml at {:?}", top_level_cargo);
            WorkspaceError::IoError {
                io_error: Arc::new(io_err),
                context: format!("reading top-level {:?}", top_level_cargo),
            }
        })?;

        let doc_top = match top_level_str.parse::<toml_edit::Document>() {
            Ok(d) => {
                trace!("Successfully parsed top-level Cargo.toml for workspace check.");
                d
            }
            Err(e) => {
                error!("Parse error in top-level Cargo.toml: {:?}", e);
                // Convert self.as_ref() to a PathBuf to avoid lifetime trouble
                let ws_path = self.as_ref().to_path_buf();
                return Err(WorkspaceError::InvalidWorkspace {
                    invalid_workspace_path: ws_path,
                });
            }
        };

        if let Some(ws_tbl) = doc_top.get("workspace").and_then(|i| i.as_table()) {
            if let Some(members_arr) = ws_tbl.get("members").and_then(|m| m.as_array()) {
                for mem_val in members_arr.iter() {
                    if let Some(rel_path) = mem_val.as_str() {
                        let cargo_toml_path = self.as_ref().join(rel_path).join("Cargo.toml");
                        let mut found = false;

                        for arc_crate in self.crates() {
                            // Lock to get crate's directory path
                            let locked = arc_crate.lock().expect("mutex lock (bump_all-check)");
                            let crate_dir = locked.as_ref().to_path_buf();
                            drop(locked);

                            if crate_dir.join("Cargo.toml") == cargo_toml_path {
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            error!(
                                "Workspace member '{}' not loaded; cargo toml at {:?} is missing/unreadable",
                                rel_path, cargo_toml_path
                            );
                            return Err(WorkspaceError::BumpError {
                                crate_path: cargo_toml_path.clone(),
                                source: Box::new(CrateError::IoError {
                                    context: format!(
                                        "reading {cargo_toml_path:?} for workspace member '{rel_path}'"
                                    ),
                                    io_error: Arc::new(std::io::Error::new(
                                        std::io::ErrorKind::NotFound,
                                        format!("No such file: {cargo_toml_path:?}"),
                                    )),
                                }),
                            });
                        }
                    }
                }
            }
        }

        // -------------------------------------------------------------
        // 2) First pass: bump each crate individually
        // -------------------------------------------------------------
        let mut updated_versions = HashMap::<String, String>::new();

        for arc_crate in self.crates() {
            // Lock once to get path + name from the crate handle
            let (crate_dir, crate_name) = {
                let guard = arc_crate.lock().expect("mutex lock (bump_all-first-pass)");
                let dir = guard.as_ref().to_path_buf();
                let nm = guard.name().to_string();
                (dir, nm)
            };

            // Reload from disk as a fresh handle => do the bump outside the Arc lock
            let mut handle: H = {
                trace!("Reloading crate handle for {:?}", crate_dir);

                // E0283 fix: we need an explicit typed intermediate
                let typed_p: P = crate_dir.clone().into();
                H::new(&typed_p).await.map_err(|err| {
                    error!("Error re-loading crate at {:?}: {:?}", crate_dir, err);
                    WorkspaceError::BumpError {
                        crate_path: crate_dir.clone(),
                        source: Box::new(err),
                    }
                })?
            };

            // Perform the bump
            handle.bump(release.clone()).await.map_err(|err| {
                error!("Error bumping crate '{}' at {:?}: {:?}", crate_name, crate_dir, err);
                WorkspaceError::BumpError {
                    crate_path: crate_dir.clone(),
                    source: Box::new(err),
                }
            })?;

            // Store the updated handle back
            {
                let mut guard = arc_crate.lock().expect("mutex lock to store updated handle");
                *guard = handle;
            }

            // Now read the new version from disk to track for second-pass updates
            let cargo_toml_path = crate_dir.join("Cargo.toml");
            trace!("Reading back new version from {:?}", cargo_toml_path);
            if let Ok(contents) = fs::read_to_string(&cargo_toml_path).await {
                if let Ok(doc) = contents.parse::<toml_edit::Document>() {
                    if let Some(pkg_tbl) = doc.get("package").and_then(|it| it.as_table()) {
                        if let Some(ver_item) = pkg_tbl.get("version").and_then(|x| x.as_str()) {
                            updated_versions.insert(crate_name, ver_item.to_string());
                        }
                    }
                }
            }
        }

        info!("All crates bumped successfully; starting second-pass reference updates.");

        // -------------------------------------------------------------
        // 3) Second pass: rewrite references in Cargo.toml for each crate
        // -------------------------------------------------------------
        for arc_crate in self.crates_mut() {
            // Lock to find the crate path
            let crate_dir = {
                let locked = arc_crate.lock().expect("mutex lock (bump_all-second-pass)");
                locked.as_ref().to_path_buf()
            };
            let cargo_toml_path = crate_dir.join("Cargo.toml");

            let contents = match fs::read_to_string(&cargo_toml_path).await {
                Ok(s) => s,
                Err(e) => {
                    warn!("Skipping second pass for {:?}: read error: {}", cargo_toml_path, e);
                    continue;
                }
            };

            let mut doc = match contents.parse::<toml_edit::Document>() {
                Ok(d) => d,
                Err(e) => {
                    warn!(
                        "Skipping second pass for {:?}: parse error: {}",
                        cargo_toml_path, e
                    );
                    continue;
                }
            };

            let mut changed = false;
            for deps_key in &["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(deps_tbl) = doc.get_mut(*deps_key).and_then(|i| i.as_table_mut()) {
                    for (dep_name, new_ver) in &updated_versions {
                        if let Some(dep_item) = deps_tbl.get_mut(dep_name) {
                            // Possibly convert inline table => normal table
                            if let Some(inline_tbl) = dep_item.as_inline_table_mut() {
                                let expanded = inline_tbl.clone().into_table();
                                *dep_item = toml_edit::Item::Table(expanded);
                            }
                            // Overwrite the version field
                            if dep_item.is_table() {
                                dep_item
                                    .as_table_mut()
                                    .unwrap()
                                    .insert("version", toml_edit::value(new_ver.clone()));
                                changed = true;
                                trace!(
                                    "Updated table-style dependency '{}' => version={}",
                                    dep_name,
                                    new_ver
                                );
                            } else if dep_item.is_str() {
                                *dep_item = toml_edit::value(new_ver.clone());
                                changed = true;
                                // Removed the extra braces that caused the error
                                trace!(
                                    "Updated string-style dependency '{}' => version={}",
                                    dep_name,
                                    new_ver
                                );
                            }
                        }
                    }
                }
            }

            if changed {
                debug!("Rewriting references in {:?}", cargo_toml_path);
                if let Err(e) = fs::write(&cargo_toml_path, doc.to_string()).await {
                    warn!("Could not rewrite references in {:?}: {}", cargo_toml_path, e);
                }
            }
        }

        info!("Finished bump_all successfully for release={:?}", release);
        Ok(())
    }
}

// ====================[ Tests changed in this same file: test_bump_all ]====================

#[cfg(test)]
mod test_bump_all {
    use super::*;
    use std::collections::HashMap;
    use tokio::fs;
    use toml_edit::Document as TomlEditDocument;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use workspacer_mock::*;
    use tracing::*;

    /// Helper: reads the `[package].version` in a crate's `Cargo.toml`.
    async fn read_version(cargo_toml: &Path) -> Option<String> {
        let contents = fs::read_to_string(cargo_toml).await.ok()?;
        let doc = contents.parse::<TomlEditDocument>().ok()?;
        let pkg_tbl = doc.get("package")?.as_table()?;
        let ver_item = pkg_tbl.get("version")?;
        ver_item.as_str().map(|s| s.to_string())
    }

    /// Helper: writes a local path dependency with a given initial version
    async fn add_local_dep_with_version(
        root_path: &Path,
        dependent_crate: &str,
        dep_crate: &str,
        version_str: &str,
    ) {
        let cargo_path = root_path.join(dependent_crate).join("Cargo.toml");
        let original = fs::read_to_string(&cargo_path)
            .await
            .expect("Failed to read dependent crate cargo toml");
        let appended = format!(
            r#"{original}
[dependencies]
{dep_crate} = {{ path = "../{dep_crate}", version="{version_str}" }}
"#
        );
        fs::write(&cargo_path, appended)
            .await
            .expect("Failed to write local dep to dependent crate's Cargo.toml");
    }

    #[traced_test]
    async fn test_bump_all_no_cross_deps() -> Result<(), CrateError> {
        info!("test_bump_all_no_cross_deps: creating crates with no inter-deps");
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let crate_c = CrateConfig::new("crate_c").with_src_files();

        let workspace_root = create_mock_workspace(vec![crate_a, crate_b, crate_c])
            .await
            .expect("Failed to create mock workspace");
        let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&workspace_root)
            .await
            .expect("Workspace creation ok");

        for arc_ch in workspace.crates() {
            let (path, name) = {
                let lock = arc_ch.lock().unwrap();
                (lock.as_ref().join("Cargo.toml"), lock.name())
            };
            let version = read_version(&path).await.unwrap();
            assert_eq!(
                version, "0.1.0",
                "{} should start at 0.1.0",
                name
            );
        }

        // Bump => Patch => 0.1.0 => 0.1.1
        info!("Calling bump_all => Patch ...");
        workspace
            .bump_all(ReleaseType::Patch)
            .await
            .expect("bump_all patch should succeed");

        for arc_ch in workspace.crates() {
            let (path, name) = {
                let lock = arc_ch.lock().unwrap();
                (lock.as_ref().join("Cargo.toml"), lock.name())
            };
            let new_ver = read_version(&path).await.unwrap();
            assert_eq!(
                new_ver, "0.1.1",
                "{} should have 0.1.1 after patch bump",
                name
            );
        }
        Ok(())
    }

    #[traced_test]
    async fn test_bump_all_with_cross_deps() -> Result<(), CrateError> {
        info!("test_bump_all_with_cross_deps: creating crates with cross-deps (a->b->c, etc)");
        // Suppose crate_a depends on crate_b, crate_b depends on crate_c
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let crate_c = CrateConfig::new("crate_c").with_src_files();

        let root_path = create_mock_workspace(vec![crate_a, crate_b, crate_c])
            .await
            .expect("Failed to create mock workspace");

        // a->b
        add_local_dep_with_version(&root_path, "crate_a", "crate_b", "0.1.0").await;
        // b->c
        add_local_dep_with_version(&root_path, "crate_b", "crate_c", "0.1.0").await;

        let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&root_path)
            .await
            .expect("workspace creation ok");

        // Bump all => Minor => 0.2.0
        workspace
            .bump_all(ReleaseType::Minor)
            .await
            .expect("bump_all minor ok");

        // Check each => now "0.2.0"
        for arc_ch in workspace.crates() {
            let (toml_path, name) = {
                let guard = arc_ch.lock().unwrap();
                (guard.as_ref().join("Cargo.toml"), guard.name())
            };
            let ver = read_version(&toml_path).await.unwrap();
            assert_eq!(ver, "0.2.0", "{} should be 0.2.0 now", name);

            // Also check references to see if they updated
            let content = fs::read_to_string(&toml_path).await.unwrap();
            let doc = content.parse::<TomlEditDocument>().unwrap();
            for deps_key in &["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(tbl) = doc.get(*deps_key).and_then(|it| it.as_table()) {
                    for (dep_name, item) in tbl.iter() {
                        // If referencing a local crate in the workspace, should be "0.2.0"
                        if workspace.find_crate_by_name(dep_name).is_some() {
                            if item.is_table() {
                                let t = item.as_table().unwrap();
                                if let Some(ver_item) = t.get("version") {
                                    assert_eq!(
                                        ver_item.as_str(),
                                        Some("0.2.0"),
                                        "dep {} => version should be 0.2.0",
                                        dep_name
                                    );
                                }
                            } else if item.is_str() {
                                assert_eq!(
                                    item.as_str(),
                                    Some("0.2.0"),
                                    "string dep {} => version should be 0.2.0",
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[traced_test]
    async fn test_bump_all_with_one_crate_failing() -> Result<(), WorkspaceError> {
        info!("test_bump_all_with_one_crate_failing");
        // 1) Create a mock workspace with two crates: crate_x and crate_y
        let crate_x = CrateConfig::new("crate_x").with_src_files();
        let crate_y = CrateConfig::new("crate_y").with_src_files();
        let root_path = create_mock_workspace(vec![crate_x, crate_y])
            .await
            .expect("Failed to create mock workspace with crate_x and crate_y");

        // 2) sabotage: remove crate_y's Cargo.toml => triggers IoError upon bump()
        let y_cargo_toml = root_path.join("crate_y").join("Cargo.toml");
        fs::remove_file(&y_cargo_toml).await.unwrap();

        // 3) Build a workspace
        let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&root_path)
            .await
            .expect("Workspace creation ok");

        // 4) Attempt to bump_all => expect crate_y to fail
        let result = workspace.bump_all(ReleaseType::Patch).await;

        match result {
            Err(WorkspaceError::BumpError { crate_path, source }) => {
                let missing_str = "crate_y/Cargo.toml".replace('/', &std::path::MAIN_SEPARATOR.to_string());
                assert!(
                    crate_path.ends_with(&missing_str),
                    "Expected failing path = crate_y’s Cargo.toml, got: {:?}",
                    crate_path
                );
                match *source {
                    CrateError::IoError { ref context, .. } => {
                        assert!(
                            context.contains("reading"),
                            "Should mention reading cargo toml in context: got {context}"
                        );
                    }
                    other => panic!("Expected CrateError::IoError, got {:?}", other),
                }
            }
            other => panic!("Expected BumpError, got {:?}", other),
        }
        Ok(())
    }

    #[traced_test]
    async fn test_bump_all_ignores_non_workspace_crates() -> Result<(), CrateError> {
        info!("test_bump_all_ignores_non_workspace_crates");
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let root = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("mock workspace creation");

        // Add external crate reference in crate_a => e.g. "serde" = "1.0"
        let a_cargo_path = root.join("crate_a").join("Cargo.toml");
        let orig_a = fs::read_to_string(&a_cargo_path).await.unwrap();
        let appended = format!(
            r#"{orig_a}
[dependencies]
serde = "1.0"
crate_b = {{ path="../crate_b", version="0.1.0" }}
"#
        );
        fs::write(&a_cargo_path, appended).await.unwrap();

        let mut ws = Workspace::<PathBuf, CrateHandle>::new(&root)
            .await
            .expect("workspace ok");

        // do bump_all => Patch => crate_a, crate_b => "0.1.0" => "0.1.1"
        ws.bump_all(ReleaseType::Patch).await.unwrap();

        // check crate_a => 0.1.1
        let a_ver = read_version(&a_cargo_path).await.unwrap();
        assert_eq!(a_ver, "0.1.1", "crate_a version => 0.1.1 after patch");

        // check crate_b => 0.1.1
        let arc_b = ws.find_crate_by_name("crate_b").unwrap();
        let b_path = {
            let locked = arc_b.lock().unwrap();
            locked.as_ref().join("Cargo.toml")
        };
        let b_ver = read_version(&b_path).await.unwrap();
        assert_eq!(b_ver, "0.1.1", "crate_b => 0.1.1 after patch");

        // check crate_a references => crate_b => "0.1.1"; serde => "1.0" unchanged
        let new_a_contents = fs::read_to_string(&a_cargo_path).await.unwrap();
        let doc_a = new_a_contents.parse::<TomlEditDocument>().unwrap();
        let deps_table = doc_a["dependencies"].as_table().unwrap();

        // crate_b => "0.1.1"
        let crate_b_dep = &deps_table["crate_b"];
        assert!(crate_b_dep.is_table());
        let c_b_ver = crate_b_dep.as_table().unwrap()["version"].as_str().unwrap();
        assert_eq!(c_b_ver, "0.1.1");

        // serde => "1.0"
        let serde_dep = &deps_table["serde"];
        assert!(serde_dep.is_str());
        assert_eq!(serde_dep.as_str(), Some("1.0"));

        Ok(())
    }

    #[traced_test]
    async fn test_bump_all_parse_error_in_second_pass() -> Result<(), WorkspaceError> {
        info!("test_bump_all_parse_error_in_second_pass");
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let root = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("Failed to create workspace with crate_a & crate_b");

        let mut ws = Workspace::<PathBuf, CrateHandle>::new(&root)
            .await
            .expect("Workspace creation ok");

        // First bump => from 0.1.0 => 0.1.1 (both crates).
        ws.bump_all(ReleaseType::Patch)
            .await
            .expect("first bump_all ok");

        // sabotage crate_a => put nonsense in [dependencies]
        let a_cargo_path = root.join("crate_a").join("Cargo.toml");
        let mut content_a = fs::read_to_string(&a_cargo_path).await.unwrap();
        content_a.push_str("\n[dependencies]\nTHIS??IS??INVALID = ??");
        fs::write(&a_cargo_path, content_a).await.unwrap();

        // second bump => 0.1.2 => but second pass parse for crate_a fails => we skip references
        let result = ws.bump_all(ReleaseType::Patch).await;
        assert!(
            result.is_ok(),
            "We expect success ignoring partial parse error in second pass"
        );

        // confirm final versions => 0.1.2 if first pass succeeded on [package]
        let ver_a = read_version(&a_cargo_path).await.unwrap_or_else(|| "???".to_string());
        let arc_b = ws.find_crate_by_name("crate_b").unwrap();
        let b_path = {
            let lb = arc_b.lock().unwrap();
            lb.as_ref().join("Cargo.toml")
        };
        let ver_b = read_version(&b_path).await.unwrap_or_else(|| "???".to_string());

        assert_eq!(ver_a, "0.1.2", "crate_a => 0.1.2");
        assert_eq!(ver_b, "0.1.2", "crate_b => 0.1.2");
        Ok(())
    }

    #[traced_test]
    async fn test_bump_all_with_build_metadata() -> Result<(), WorkspaceError> {
        info!("test_bump_all_with_build_metadata");
        let crate_x = CrateConfig::new("crate_x").with_src_files();
        let root = create_mock_workspace(vec![crate_x])
            .await
            .expect("workspace creation ok");
        let mut ws = Workspace::<PathBuf, CrateHandle>::new(&root)
            .await
            .expect("ws ok");

        // sabotage: set crate_x => version="0.1.0+build999"
        let cargo_x_path = root.join("crate_x").join("Cargo.toml");
        let contents = fs::read_to_string(&cargo_x_path).await.unwrap();
        let mut doc = contents.parse::<TomlEditDocument>().unwrap();
        {
            let pkg_tbl = doc["package"].as_table_mut().unwrap();
            pkg_tbl.insert("version", toml_edit::value("0.1.0+build999"));
        }
        fs::write(&cargo_x_path, doc.to_string()).await.unwrap();

        // do bump_all => Patch => "0.1.0+build999" => "0.1.1+build999"
        ws.bump_all(ReleaseType::Patch).await.unwrap();
        let updated = read_version(&cargo_x_path).await.unwrap();
        assert_eq!(updated, "0.1.1+build999");
        Ok(())
    }
}
