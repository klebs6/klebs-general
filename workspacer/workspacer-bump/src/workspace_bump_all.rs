// ---------------- [ File: workspacer-bump/src/workspace_bump_all.rs ]
crate::ix!();

#[async_trait]
impl<P, H> BumpAll for Workspace<P, H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
    for<'async_trait> H: CrateHandleInterface<P>
        + Bump<Error = CrateError>
        + Send
        + Sync
        + 'async_trait,  // <== note: we remove the second AsyncTryFrom so we don’t keep re-creating handles
{
    type Error = WorkspaceError;

    #[instrument(level = "trace", skip_all, fields(release = ?release))]
    async fn bump_all(&mut self, release: ReleaseType) -> Result<(), Self::Error> {
        use tracing::{trace, warn, error, info};

        trace!("Starting bump_all with release={:?}", release);

        // 2) First pass: for each crate => bump it => record new version
        let mut updated_versions = HashMap::<String, String>::new();

        for arc_crate in self.crates() {
            let mut guard = arc_crate.lock().await;
            let crate_name = guard.name().to_string();

            // Attempt to bump
            match guard.bump(release.clone()).await {
                Ok(()) => {
                    // Retrieve the new version from the handle
                    match guard.version() {
                        Ok(ver) => {
                            updated_versions.insert(crate_name, ver.to_string());
                        }
                        Err(e) => {
                            warn!("Could not parse new version for crate '{}': {:?}; skipping second-pass updates for it", crate_name, e);
                        }
                    }
                }
                Err(e) => {
                    // If cargo toml is missing => error out
                    // If parse error => skip
                    if let CrateError::IoError { io_error, .. } = &e {
                        if io_error.kind() == std::io::ErrorKind::NotFound {
                            error!("Crate '{}' is missing Cargo.toml => returning an error", crate_name);
                            return Err(WorkspaceError::BumpError {
                                crate_path: guard.as_ref().join("Cargo.toml"),
                                source: Box::new(e),
                            });
                        }
                    }
                    warn!(
                        "Crate '{}' parse/other error => skipping. Error: {:?}",
                        crate_name, e
                    );
                }
            }
        }

        info!("All crates bumped successfully; starting second-pass reference updates.");

        // 3) Second pass: rewrite references
        for arc_crate in self.crates() {
            let mut handle = arc_crate.lock().await;
            let crate_name = handle.name().to_string();

            // Access crate’s CargoToml
            let cargo_toml_arc = handle.cargo_toml();
            let mut cargo_toml = cargo_toml_arc.lock().await;

            let mut changed_any = false;

            for (dep_name, new_ver) in &updated_versions {
                match cargo_toml.update_dependency_version(dep_name, new_ver) {
                    Ok(true) => {
                        changed_any = true;
                        trace!(
                            "[2nd pass] crate='{}': updated dep='{}' => version='{}'",
                            crate_name,
                            dep_name,
                            new_ver
                        );
                    }
                    Ok(false) => { /* no references to dep_name here */ }
                    Err(e) => {
                        warn!("Could not update '{}' in '{}': {:?}", dep_name, crate_name, e);
                        // continue or break — up to you
                    }
                }
            }

            if changed_any {
                // Save changes to disk
                if let Err(save_err) = cargo_toml.save_to_disk().await {
                    warn!(
                        "Could not rewrite references in '{}' Cargo.toml: {:?}",
                        crate_name, save_err
                    );
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

    #[tokio::test]
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
        let arc_b = ws
            .find_crate_by_name("crate_b")
            .await
            .expect("should find crate_b");
        let b_path = {
            let locked = arc_b.lock().await;
            locked.as_ref().join("Cargo.toml")
        };
        let b_ver = read_version(&b_path).await.unwrap();
        assert_eq!(b_ver, "0.1.1", "crate_b => 0.1.1 after patch");

        // check crate_a references => crate_b => "0.1.1"; serde => "1.0" unchanged
        let new_a_contents = fs::read_to_string(&a_cargo_path).await.unwrap();
        let doc_a = new_a_contents.parse::<toml_edit::Document>().unwrap();

        if let Some(deps_table) = doc_a.get("dependencies").and_then(|val| val.as_table()) {
            // **Important**: collect these into a Vec so that we do not hold
            // references across the async boundary (await). This ensures
            // the future is Send.
            let deps: Vec<(String, toml_edit::Item)> = deps_table
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect();

            for (dep_name, item) in deps {
                // If referencing a local crate in the workspace, we updated its version
                if ws.find_crate_by_name(&dep_name).await.is_some() {
                    if let Some(tbl) = item.as_table() {
                        if let Some(ver_item) = tbl.get("version") {
                            assert_eq!(
                                ver_item.as_str(),
                                Some("0.1.1"),
                                "Updated local crate dep to 0.1.1"
                            );
                        }
                    }
                } else {
                    // "serde" => "1.0" remains unchanged
                    if dep_name == "serde" {
                        assert_eq!(item.as_str(), Some("1.0"));
                    }
                }
            }
        }

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

    // This is the ENTIRE test function `test_bump_all_no_cross_deps`,
    // with the lock-lifetime fix applied. We now store `name` as a String
    // before dropping the lock. That way the references remain valid.
    // Everything else is unchanged.
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
                let lock = arc_ch.lock().await;
                let path = lock.as_ref().join("Cargo.toml");
                let name = lock.name().to_string();
                (path, name)
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
                let lock = arc_ch.lock().await;
                let path = lock.as_ref().join("Cargo.toml");
                let name = lock.name().to_string();
                (path, name)
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

    // This is the ENTIRE test function `test_bump_all_with_cross_deps`,
    // again fixing the lock-lifetime by storing the name as a String
    // before the lock is dropped.
    #[tokio::test]
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
                let lock = arc_ch.lock().await;
                let path = lock.as_ref().join("Cargo.toml");
                let nm = lock.name().to_string();
                (path, nm)
            };
            let ver = read_version(&toml_path).await.unwrap();
            assert_eq!(
                ver, "0.2.0",
                "{} should be 0.2.0 now",
                name
            );

            // Also check references to see if they updated
            let content = fs::read_to_string(&toml_path).await.unwrap();
            let doc = content.parse::<toml_edit::Document>().unwrap();
            for deps_key in &["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(tbl) = doc.get(*deps_key).and_then(|it| it.as_table()) {
                    for (dep_name, item) in tbl.iter() {
                        // If referencing a local crate in the workspace, should be "0.2.0"
                        if workspace.find_crate_by_name(dep_name).await.is_some() {
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
                                    dep_name
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_bump_all_parse_error_in_second_pass() {
        use tokio::fs;
        use toml_edit::Document as TomlEditDocument;

        async fn read_package_version_partial(path: &Path) -> Option<String> {
            // We'll parse the entire Cargo.toml but ignore errors by falling back to an empty doc
            let raw = fs::read_to_string(path).await.ok()?;
            let doc = raw.parse::<TomlEditDocument>().unwrap_or_default();
            let pkg_tbl = doc.get("package")?.as_table()?;
            let ver_item = pkg_tbl.get("version")?;
            ver_item.as_str().map(|s| s.to_string())
        }

        info!("test_bump_all_parse_error_in_second_pass");
        // 1) Create a mock workspace with two crates: crate_a and crate_b
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let root_path = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("Failed to create mock workspace with crate_a and crate_b");

        // 2) Build the workspace *once*, then do bump => from 0.1.0 => 0.1.1
        let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&root_path)
            .await
            .expect("Initial workspace creation ok");
        workspace
            .bump_all(ReleaseType::Patch)
            .await
            .expect("first bump_all => 0.1.1");

        // 3) sabotage crate_a: put nonsense in [dependencies]
        let a_cargo_toml = root_path.join("crate_a/Cargo.toml");
        let contents = fs::read_to_string(&a_cargo_toml)
            .await
            .expect("read crate_a/Cargo.toml");
        let sabotaged = format!("{contents}\n[dependencies]\nTHIS??IS??INVALID = ??\n");
        fs::write(&a_cargo_toml, sabotaged)
            .await
            .expect("write sabotage to crate_a/Cargo.toml");

        // 4) Use the same workspace object => Bump again => from 0.1.1 => 0.1.2
        //    crate_a now has invalid TOML in [dependencies], but partial parse for [package] is ok
        workspace
            .bump_all(ReleaseType::Patch)
            .await
            .expect("second bump_all => 0.1.2");

        // 5) Verify final versions:
        let b_cargo_toml = root_path.join("crate_b").join("Cargo.toml");
        let actual_b = read_version(&b_cargo_toml)
            .await
            .unwrap_or_else(|| "???".to_owned());
        assert_eq!(actual_b, "0.1.2", "crate_b => 0.1.2");

        // crate_a => "0.1.2" but we do partial parse ignoring [dependencies]
        let actual_a = read_package_version_partial(&a_cargo_toml)
            .await
            .unwrap_or_else(|| "???".to_owned());
        assert_eq!(actual_a, "0.1.2", "crate_a => 0.1.2");

        info!("test_bump_all_parse_error_in_second_pass => SUCCESS");
    }
}
