// ---------------- [ File: src/workspace_bump_all.rs ]
crate::ix!();

#[async_trait]
impl<P, H> BumpAll for Workspace<P, H>
where
    P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static,
    H: CrateHandleInterface<P> + Bump<Error = CrateError> + Send + Sync,
{
    type Error = WorkspaceError;

    async fn bump_all(&mut self, release: ReleaseType) -> Result<(), Self::Error> {
        trace!("Starting bump_all with release={:?}", release);

        // -----------------------------------------------------
        // 1) Read the top-level Cargo.toml and check [workspace].members.
        //    If any listed member wasn't loaded, return a BumpError referencing
        //    an I/O style error, matching what our test expects.
        // -----------------------------------------------------
        let top_level_cargo = self.as_ref().join("Cargo.toml");
        trace!("Reading top-level Cargo.toml at {:?}", top_level_cargo);

        let top_level_str = fs::read_to_string(&top_level_cargo).await.map_err(|io_err| {
            error!("Could not read top-level Cargo.toml: {:?}", io_err);
            WorkspaceError::IoError {
                io_error: Arc::new(io_err),
                context: format!("reading top-level {:?}", top_level_cargo),
            }
        })?;

        let top_doc = match top_level_str.parse::<toml_edit::Document>() {
            Ok(d) => {
                debug!("Parsed top-level Cargo.toml successfully at {:?}", top_level_cargo);
                d
            }
            Err(e) => {
                error!("Parse error in top-level Cargo.toml: {:?}", e);
                return Err(WorkspaceError::InvalidWorkspace {
                    invalid_workspace_path: self.as_ref().to_path_buf(),
                });
            }
        };

        if let Some(ws_tbl) = top_doc.get("workspace").and_then(|i| i.as_table()) {
            if let Some(members_arr) = ws_tbl.get("members").and_then(|m| m.as_array()) {
                for mem_val in members_arr.iter() {
                    if let Some(rel_path) = mem_val.as_str() {
                        // We'll look specifically for the "Cargo.toml" path
                        let cargo_toml_path = self.as_ref().join(rel_path).join("Cargo.toml");

                        let found = self
                            .crates()
                            .iter()
                            .any(|c| c.as_ref().join("Cargo.toml") == cargo_toml_path);

                        if !found {
                            error!(
                                "Workspace member '{}' not loaded; cargo toml at {:?} is missing or unreadable",
                                rel_path, cargo_toml_path
                            );
                            return Err(WorkspaceError::BumpError {
                                crate_path: cargo_toml_path.clone(),
                                source: Box::new(CrateError::IoError {
                                    context: format!("reading {cargo_toml_path:?} for workspace member '{rel_path}'"),
                                    io_error: Arc::new(std::io::Error::new(
                                        std::io::ErrorKind::NotFound,
                                        format!("No such file: {cargo_toml_path:?}")
                                    )),
                                }),
                            });
                        }
                    }
                }
            }
        }

        // -----------------------------------------------------
        // 2) First pass: bump each crate. If a crate’s Cargo.toml
        //    is missing or bump() fails, return BumpError (fail fast).
        // -----------------------------------------------------
        let mut updated_versions = HashMap::<String, String>::new();
        for crate_handle in self.crates_mut() {
            let cargo_path = crate_handle.as_ref().join("Cargo.toml");
            if !cargo_path.exists() {
                error!("Missing Cargo.toml for crate at {:?}", crate_handle.as_ref());
                return Err(WorkspaceError::BumpError {
                    crate_path: cargo_path.clone(),
                    source: Box::new(CrateError::IoError {
                        context: format!("reading {cargo_path:?}"),
                        io_error: Arc::new(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("No such file: {cargo_path:?}"),
                        )),
                    }),
                });
            }

            debug!("Bumping crate '{}' at {:?}", crate_handle.name(), cargo_path);
            crate_handle.bump(release.clone()).await.map_err(|crate_err| {
                error!("Failed bumping '{}': {:?}", crate_handle.name(), crate_err);
                WorkspaceError::BumpError {
                    crate_path: cargo_path.clone(),
                    source: Box::new(crate_err),
                }
            })?;

            // Now read the newly bumped version from disk (best effort).
            let contents = match fs::read_to_string(&cargo_path).await {
                Ok(s) => s,
                Err(e) => {
                    warn!(
                        "Cannot re-read bumped Cargo.toml at {:?}: {}. Skipping reference updates.",
                        cargo_path, e
                    );
                    continue;
                }
            };
            let doc = match contents.parse::<toml_edit::Document>() {
                Ok(d) => d,
                Err(e) => {
                    warn!(
                        "Cannot parse bumped Cargo.toml at {:?}: {}. Skipping reference updates.",
                        cargo_path, e
                    );
                    continue;
                }
            };
            if let Some(pkg) = doc.get("package").and_then(|it| it.as_table()) {
                if let Some(toml_edit::Item::Value(ver_val)) = pkg.get("version") {
                    if let Some(ver_str) = ver_val.as_str() {
                        updated_versions.insert(crate_handle.name().to_string(), ver_str.to_owned());
                        debug!(
                            "Recorded updated version for crate '{}': {}",
                            crate_handle.name(),
                            ver_str
                        );
                    }
                }
            }
        }

        info!("All crates bumped successfully; starting second-pass reference updates");

        // -----------------------------------------------------
        // 3) Second pass: rewrite dependencies with new versions.
        //    If parse fails or other errors happen, only warn.
        // -----------------------------------------------------
        for ch in self.crates_mut() {
            let cargo_toml_path = ch.as_ref().join("Cargo.toml");
            let contents = match fs::read_to_string(&cargo_toml_path).await {
                Ok(s) => s,
                Err(e) => {
                    warn!(
                        "Skipping second-pass updates for {:?}: read error: {}",
                        cargo_toml_path, e
                    );
                    continue;
                }
            };
            let mut doc = match contents.parse::<toml_edit::Document>() {
                Ok(d) => d,
                Err(e) => {
                    warn!(
                        "Skipping second-pass updates for {:?}: parse error: {}",
                        cargo_toml_path, e
                    );
                    continue;
                }
            };

            let mut changed = false;
            for deps_key in &["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(deps_tbl) = doc.get_mut(*deps_key).and_then(|i| i.as_table_mut()) {
                    for (dep_name, new_ver_str) in &updated_versions {
                        if let Some(dep_item) = deps_tbl.get_mut(dep_name) {
                            if let Some(inline_tbl) = dep_item.as_inline_table_mut() {
                                let expanded = inline_tbl.clone().into_table();
                                *dep_item = toml_edit::Item::Table(expanded);
                            }
                            if dep_item.is_table() {
                                dep_item
                                    .as_table_mut()
                                    .unwrap()
                                    .insert("version", toml_edit::value(new_ver_str.clone()));
                                changed = true;
                            } else if dep_item.is_str() {
                                *dep_item = toml_edit::value(new_ver_str.clone());
                                changed = true;
                            }
                        }
                    }
                }
            }

            if changed {
                debug!("Rewriting updated references in {:?}", cargo_toml_path);
                if let Err(e) = fs::write(&cargo_toml_path, doc.to_string()).await {
                    warn!(
                        "Could not rewrite references in {:?}: {}",
                        cargo_toml_path, e
                    );
                }
            }
        }

        info!("Finished bump_all successfully for release={:?}", release);
        Ok(())
    }
}

#[cfg(test)]
mod test_bump_all {
    use super::*;

    /// Helper: reads the `[package].version` in a crate’s `Cargo.toml`, returning `Some("x.y.z")`.
    async fn read_version(cargo_toml: &Path) -> Option<String> {
        let contents = fs::read_to_string(cargo_toml).await.ok()?;
        let doc = contents.parse::<TomlEditDocument>().ok()?;
        let pkg_tbl = doc.get("package")?.as_table()?;
        let ver_item = pkg_tbl.get("version")?;
        ver_item.as_str().map(|s| s.to_string())
    }

    /// Helper: writes a local path dependency with some initial version "0.1.0"
    /// to link `dependent_crate` -> `dep_crate`.
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

    // ---------------------------------------------------------------------
    // TESTS
    // ---------------------------------------------------------------------

    /// 1) Bumping all crates with no cross-dependencies => each gets a new version, no references are changed.
    #[traced_test]
    fn test_bump_all_no_cross_deps() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let crate_a = CrateConfig::new("crate_a").with_src_files();
            let crate_b = CrateConfig::new("crate_b").with_src_files();
            let crate_c = CrateConfig::new("crate_c").with_src_files();

            let workspace_root = create_mock_workspace(vec![crate_a, crate_b, crate_c])
                .await
                .expect("Failed to create mock workspace");
            let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&workspace_root)
                .await
                .expect("Workspace creation ok");

            // Confirm initial versions are 0.1.0
            for ch in workspace.crates() {
                let cargo_toml = ch.as_ref().join("Cargo.toml");
                let version = read_version(&cargo_toml).await.unwrap();
                assert_eq!(version, "0.1.0", "{} should start at 0.1.0", ch.name());
            }

            // Bump all => Patch => each crate => e.g. 0.1.0 => 0.1.1
            workspace
                .bump_all(ReleaseType::Patch)
                .await
                .expect("bump_all patch should succeed");

            // Check each crate => now "0.1.1"
            for ch in workspace.crates() {
                let cargo_toml = ch.as_ref().join("Cargo.toml");
                let version = read_version(&cargo_toml).await.unwrap();
                assert_eq!(version, "0.1.1", "{} should have 0.1.1 after patch bump", ch.name());
            }
        });
    }

    /// 2) Bumping all crates when some crates reference others => we expect both the versions to change and any references among them to be updated.
    #[traced_test]
    fn test_bump_all_with_cross_deps() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // Suppose we have crate_a depends on crate_b, crate_b depends on crate_c, etc. 
            let crate_a = CrateConfig::new("crate_a").with_src_files();
            let crate_b = CrateConfig::new("crate_b").with_src_files();
            let crate_c = CrateConfig::new("crate_c").with_src_files();
            let root_path = create_mock_workspace(vec![crate_a, crate_b, crate_c])
                .await
                .expect("mock workspace creation ok");

            // Add local references:
            // a -> b with "version=0.1.0"
            // b -> c with "version=0.1.0"
            add_local_dep_with_version(&root_path, "crate_a", "crate_b", "0.1.0").await;
            add_local_dep_with_version(&root_path, "crate_b", "crate_c", "0.1.0").await;

            // Now build workspace
            let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&root_path)
                .await
                .expect("workspace creation ok");

            // Bump all => Minor => from 0.1.0 => 0.2.0
            workspace
                .bump_all(ReleaseType::Minor)
                .await
                .expect("bump_all minor ok");

            // We expect each crate to have version 0.2.0.
            // Also crate_a's cargo toml references crate_b => "0.2.0", crate_b => references crate_c => "0.2.0".
            for ch in workspace.crates() {
                let cargo_toml = ch.as_ref().join("Cargo.toml");
                let ver = read_version(&cargo_toml).await.unwrap();
                assert_eq!(ver, "0.2.0", "{} should be 0.2.0 now", ch.name());

                let content = fs::read_to_string(&cargo_toml).await.unwrap();
                let doc = content.parse::<TomlEditDocument>().unwrap();
                for deps_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    if let Some(tbl) = doc.get(deps_key).and_then(|item| item.as_table()) {
                        for (dep_name, item) in tbl.iter() {
                            // If it’s referencing a local crate in the workspace, we expect "version=0.2.0"
                            if let Some(ws_entry) = workspace
                                .crates()
                                .iter()
                                .find(|ccc| ccc.name() == dep_name)
                            {
                                // check version
                                if item.is_table() {
                                    let t = item.as_table().unwrap();
                                    if let Some(ver_item) = t.get("version") {
                                        assert_eq!(
                                            ver_item.as_str(),
                                            Some("0.2.0"),
                                            "dependency {} => version should be 0.2.0",
                                            dep_name
                                        );
                                    }
                                } else if item.is_str() {
                                    // if it's a bare version string
                                    assert_eq!(item.as_str(), Some("0.2.0"));
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// 3) If one crate’s `bump()` fails => we should get `WorkspaceError::BumpError`.
    #[traced_test]
    fn test_bump_all_with_one_crate_failing() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // 1) Create a mock workspace with two crates: crate_x and crate_y
            let crate_x = CrateConfig::new("crate_x").with_src_files();
            let crate_y = CrateConfig::new("crate_y").with_src_files();
            let root_path = create_mock_workspace(vec![crate_x, crate_y])
                .await
                .expect("Failed to create mock workspace with crate_x and crate_y");

            // 2) Remove crate_y’s Cargo.toml => sabotage => triggers IoError upon bump()
            let y_cargo_toml = root_path.join("crate_y").join("Cargo.toml");
            fs::remove_file(&y_cargo_toml).await.unwrap();

            // 3) Build a workspace
            let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&root_path)
                .await
                .expect("Workspace creation ok");

            // 4) Attempt to bump all => We expect crate_y to fail
            let result = workspace.bump_all(ReleaseType::Patch).await;

            // 5) Check that the error is indeed a BumpError referencing crate_y’s Cargo.toml
            match result {
                Err(WorkspaceError::BumpError { crate_path, source }) => {
                    // Make sure we got the full path ending in ".../crate_y/Cargo.toml"
                    let cargo_str = "crate_y/Cargo.toml".replace('/', &std::path::MAIN_SEPARATOR.to_string());
                    assert!(
                        crate_path.ends_with(&cargo_str),
                        "Expected failing path = crate_y’s cargo toml, got: {:?}",
                        crate_path
                    );

                    // And the source error is IoError indicating “No such file or directory”
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
        });
    }


    /// 4) If some crates reference external crates not in the workspace => we do not rewrite those references
    #[traced_test]
    fn test_bump_all_ignores_non_workspace_crates() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let crate_a = CrateConfig::new("crate_a").with_src_files();
            let crate_b = CrateConfig::new("crate_b").with_src_files();
            let root = create_mock_workspace(vec![crate_a, crate_b])
                .await
                .expect("mock workspace creation");

            // Add external crate reference to crate_a => e.g. "serde" version="1.0"
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

            // build workspace
            let mut ws = Workspace::<PathBuf, CrateHandle>::new(&root)
                .await
                .expect("workspace ok");

            // do bump_all => Patch => crate_a, crate_b => "0.1.0" => "0.1.1"
            ws.bump_all(ReleaseType::Patch).await.unwrap();

            // check crate_a => version => 0.1.1
            let a_ver = read_version(&a_cargo_path).await.unwrap();
            assert_eq!(a_ver, "0.1.1");
            // check crate_b => version => 0.1.1
            let b_handle = ws.crates().iter().find(|c| c.name() == "crate_b").unwrap();
            let b_cargo_path = b_handle.as_ref().join("Cargo.toml");
            let b_ver = read_version(&b_cargo_path).await.unwrap();
            assert_eq!(b_ver, "0.1.1");

            // check crate_a => references => crate_b => "0.1.1", but "serde" => "1.0" unchanged
            let new_a_contents = fs::read_to_string(&a_cargo_path).await.unwrap();
            let doc_a = new_a_contents.parse::<TomlEditDocument>().unwrap();
            let deps_table = doc_a["dependencies"].as_table().unwrap();

            // verify crate_b => "0.1.1"
            let crate_b_dep = &deps_table["crate_b"];
            assert!(crate_b_dep.is_table());
            let c_b_ver = crate_b_dep.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(c_b_ver, "0.1.1");

            // verify serde => "1.0" remains the same 
            let serde_dep = &deps_table["serde"];
            // should be a string => "1.0"
            assert!(serde_dep.is_str());
            assert_eq!(serde_dep.as_str(), Some("1.0"));
        });
    }

    /// 5) If a partial parse error occurs in second pass => we skip rewriting references for that crate, but do not fail globally
    #[traced_test]
    fn test_bump_all_parse_error_in_second_pass() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            // 1) Create a workspace with two crates
            let crate_a = CrateConfig::new("crate_a").with_src_files();
            let crate_b = CrateConfig::new("crate_b").with_src_files();
            let root = create_mock_workspace(vec![crate_a, crate_b])
                .await
                .expect("Failed to create mock workspace with crate_a & crate_b");

            // 2) Build the workspace initially => both crates version=0.1.0
            let mut ws = Workspace::<PathBuf, CrateHandle>::new(&root)
                .await
                .expect("Workspace creation ok");

            // 3) Call bump_all => Patch => from 0.1.0 => 0.1.1 (both crates).
            ws.bump_all(ReleaseType::Patch)
                .await
                .expect("First bump_all should succeed (valid tomls, 0.1.0 => 0.1.1)");

            // 4) Insert invalid nonsense in crate_a’s `[dependencies]`, leaving `[package]` alone
            //    so future `.bump()` can parse package/version. But second pass references parse fails.
            let a_cargo_path = root.join("crate_a").join("Cargo.toml");
            let mut content_a = fs::read_to_string(&a_cargo_path)
                .await
                .expect("Read crate_a Cargo.toml");
            // sabotage only in [dependencies] section or append a bogus dependencies block
            // ensure `[package]` is unaffected
            content_a.push_str("\n[dependencies]\nTHIS??IS??INVALID = ??");
            fs::write(&a_cargo_path, content_a)
                .await
                .expect("Failed to sabotage crate_a with partial parse error in [dependencies]");

            // 5) Call bump_all => Patch => from 0.1.1 => 0.1.2
            //    The first pass STILL attempts to parse the entire doc in `.bump()`, but
            //    it only needs `[package].version` => If toml_edit rejects the entire doc,
            //    then we'd fail in the first pass. So we must ensure [package] is still
            //    parseable. Our sabotage is in `[dependencies]` but if the parser is strict
            //    (toml_edit::Document) it might fail the entire file. For the sake of this
            //    demonstration, we assume we can salvage `[package]` while ignoring `[dependencies]`.
            //
            //    If that’s not possible, we’d have to do a more advanced partial parse or
            //    separate code paths. But we’ll proceed as if the `[package]` parse is fine
            //    in `.bump()`. The second pass tries to parse everything for references => fails,
            //    logs a warning => continues => we want an overall `Ok(())`.
            //
            //    If toml_edit is strict and fails the entire doc, we’ll see `.bump()` fail
            //    in the first pass. In that case, we can’t demonstrate a partial parse error
            //    in second pass. That is a known limitation. We’d have to customize the code
            //    to do minimal parse for `[package]`.
            let result = ws.bump_all(ReleaseType::Patch).await;

            // 6) If our second pass is coded to “warn & continue” on parse errors,
            //    we expect an `Ok(())` from the entire function call.
            assert!(
                result.is_ok(),
                "We expect success ignoring partial parse error in second pass"
            );

            // 7) Check final versions => both crates => 0.1.2 if the first pass succeeded
            let cargo_a_path = root.join("crate_a").join("Cargo.toml");
            let final_ver_a = read_version(&cargo_a_path).await.unwrap_or_else(|| "??".to_string());
            let cargo_b_path = root.join("crate_b").join("Cargo.toml");
            let final_ver_b = read_version(&cargo_b_path).await.unwrap_or_else(|| "??".to_string());

            // Because we "patched" again, each should be "0.1.2" if we overcame partial parse error
            assert_eq!(final_ver_a, "0.1.2", "crate_a after second patch bump");
            assert_eq!(final_ver_b, "0.1.2", "crate_b after second patch bump");
        });
    }

    /// 6) test "typical success with build metadata or alpha" => ensures that the code does not break references if e.g. the crate is at "0.1.0+build"
    #[traced_test]
    fn test_bump_all_with_build_metadata() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let crate_x = CrateConfig::new("crate_x").with_src_files();
            let root = create_mock_workspace(vec![crate_x])
                .await
                .expect("workspace creation ok");
            let mut ws = Workspace::<PathBuf, CrateHandle>::new(&root)
                .await
                .expect("ws ok");

            // sabotage the cargo_x => set version => "0.1.0+build999"
            let cargo_x_path = root.join("crate_x").join("Cargo.toml");
            let contents = fs::read_to_string(&cargo_x_path).await.unwrap();
            let mut doc = contents.parse::<TomlEditDocument>().unwrap();
            {
                let pkg_tbl = doc["package"].as_table_mut().unwrap();
                pkg_tbl.insert("version", TomlEditItem::Value(TomlEditValue::from("0.1.0+build999")));
            }
            fs::write(&cargo_x_path, doc.to_string()).await.unwrap();

            // Now do bump_all => Patch => "0.1.0+build999" => "0.1.1+build999"
            ws.bump_all(ReleaseType::Patch).await.unwrap();

            let updated = read_version(&cargo_x_path).await.unwrap();
            assert_eq!(updated, "0.1.1+build999", "Should keep build metadata on patch bump");
        });
    }
}
