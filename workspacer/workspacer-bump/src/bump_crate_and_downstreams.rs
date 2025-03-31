// ---------------- [ File: workspacer-bump/src/bump_crate_and_downstreams.rs ]
crate::ix!();

#[async_trait]
impl<P, H> BumpCrateAndDownstreams for Workspace<P, H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Bump<Error = CrateError> + Send + Sync,
{
    type Error = WorkspaceError;

    async fn bump_crate_and_downstreams(
        &mut self,
        crate_handle: &mut CrateHandle,
        release: ReleaseType
    ) -> Result<(), Self::Error> {
        trace!(
            "bump_crate_and_downstreams: bumping '{}' => release={:?}",
            crate_handle.name(),
            release
        );

        // 1) Attempt to bump
        if let Err(err) = crate_handle.bump(release.clone()).await {
            error!("Error bumping crate '{}': {:?}", crate_handle.name(), err);
            return Err(WorkspaceError::BumpError {
                crate_path: crate_handle.as_ref().join("Cargo.toml"),
                source: Box::new(err),
            });
        }

        let new_ver = crate_handle.version().map_err(|ver_err| {
            error!("Could not re-parse new version for '{}': {:?}", crate_handle.name(), ver_err);
            WorkspaceError::CrateError(ver_err)
        })?;

        info!(
            "bump_crate_and_downstreams: crate='{}' => new_ver={}",
            crate_handle.name(),
            new_ver
        );

        let crate_key = crate_handle.name().to_string();
        let mut visited = HashSet::new();
        visited.insert(crate_key.clone());

        self.update_downstreams_recursively(&crate_key, &new_ver, &mut visited).await
    }
}

#[cfg(test)]
mod test_bump_crate_and_downstreams {
    use super::*;

    async fn read_crate_version(cargo_toml_path: &Path) -> Option<String> {
        let contents = fs::read_to_string(cargo_toml_path).await.ok()?;
        let doc = contents.parse::<TomlEditDocument>().ok()?;
        let pkg = doc.get("package")?.as_table()?;
        let ver_item = pkg.get("version")?;
        ver_item.as_str().map(|s| s.to_string())
    }

    async fn setup_workspace_and_handle(
        crate_configs: Vec<CrateConfig>,
        target_crate_name: &str,
    ) -> (TempDir, Workspace<PathBuf, CrateHandle>, Arc<AsyncMutex<CrateHandle>>) {
        let tmp_root = create_mock_workspace(crate_configs)
            .await
            .expect("Failed to create mock workspace");

        let workspace = Workspace::<PathBuf, CrateHandle>::new(&tmp_root)
            .await
            .expect("Failed to create Workspace");

        let ch = workspace
            .find_crate_by_name(target_crate_name)
            .await
            .expect("expected find crate by name to succeed");

        (
            TempDir::new_in(tmp_root.parent().unwrap()).unwrap(),
            workspace,
            ch
        )
    }

    #[traced_test]
    async fn test_bump_with_no_downstreams() -> Result<(), CrateError> {
        let crate_a = CrateConfig::new("crate_a").with_src_files().with_readme();
        let crate_b = CrateConfig::new("crate_b").with_src_files().with_readme();
        let tmp_root = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("Failed to create mock workspace");

        let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&tmp_root)
            .await
            .expect("Failed to create Workspace");

        // We'll choose crate_a, which (so far) has no crates depending on it
        let arc_a = workspace.find_crate_by_name("crate_a").await
            .expect("crate_a not found in workspace");

        // Confirm initial version
        {
            let handle_clone = arc_a.lock().await.clone();
            assert_eq!(handle_clone.version()?.to_string(), "0.1.0");
        }

        // Bump => patch => "0.1.1"
        {
            let mut local_clone = {
                let guard = arc_a.lock().await;
                guard.clone()
            };
            workspace
                .bump_crate_and_downstreams(&mut local_clone, ReleaseType::Patch)
                .await
                .expect("bump crate_a patch ok");
            // store back
            {
                let mut guard = arc_a.lock().await;
                *guard = local_clone;
            }
        }

        // Confirm crate_a => 0.1.1
        {
            let path_a = {
                let guard = arc_a.lock().await;
                guard.as_ref().join("Cargo.toml")
            };
            let new_ver_a = read_crate_version(&path_a).await.unwrap();
            assert_eq!(new_ver_a, "0.1.1");
        }

        // Confirm crate_b => still "0.1.0"
        {
            let arc_b = workspace.find_crate_by_name("crate_b").await
                .expect("crate_b not found");
            let path_b = {
                let guard_b = arc_b.lock().await;
                guard_b.as_ref().join("Cargo.toml")
            };
            let ver_b = read_crate_version(&path_b).await.unwrap();
            assert_eq!(ver_b, "0.1.0", "crate_b should be unchanged");
        }
        Ok(())
    }

    #[traced_test]
    async fn test_bump_with_single_downstream() -> Result<(), CrateError> {
        let crate_a_cfg = CrateConfig::new("crate_a").with_src_files().with_readme();
        let crate_b_cfg = CrateConfig::new("crate_b").with_src_files().with_readme();
        let tmp = create_mock_workspace(vec![crate_a_cfg, crate_b_cfg])
            .await
            .expect("mock workspace creation failed");

        // a -> b
        let a_cargo_path = tmp.join("crate_a").join("Cargo.toml");
        let orig_a = fs::read_to_string(&a_cargo_path).await.unwrap();
        let appended = format!(
            r#"{}
    [dependencies]
    crate_b = {{ path = "../crate_b", version = "0.1.0" }}
    "#,
            orig_a
        );
        fs::write(&a_cargo_path, appended).await.unwrap();

        let mut ws = Workspace::<PathBuf, CrateHandle>::new(&tmp)
            .await
            .expect("workspace creation ok");

        let arc_b = ws.find_crate_by_name("crate_b").await
            .expect("crate_b not found in workspace");

        // Bump crate_b => major => "1.0.0"
        {
            let mut local_clone = {
                let gb = arc_b.lock().await;
                gb.clone()
            };
            ws.bump_crate_and_downstreams(&mut local_clone, ReleaseType::Major)
                .await
                .expect("bump crate_b major ok");
            {
                let mut gb = arc_b.lock().await;
                *gb = local_clone;
            }
        }

        // Check crate_b => "1.0.0"
        {
            let path_b = {
                let gb = arc_b.lock().await;
                gb.as_ref().join("Cargo.toml")
            };
            let ver_b = read_crate_version(&path_b).await.unwrap();
            assert_eq!(ver_b, "1.0.0");
        }

        // Check crate_a => references crate_b => "version=1.0.0"
        {
            let new_a_contents = fs::read_to_string(&a_cargo_path).await.unwrap();
            let doc_a = new_a_contents.parse::<toml_edit::Document>().unwrap();
            let deps_table = doc_a["dependencies"].as_table().unwrap();
            let crate_b_dep = &deps_table["crate_b"];
            assert!(crate_b_dep.is_table());
            let ver_item = crate_b_dep.as_table().unwrap().get("version").unwrap();
            assert_eq!(ver_item.as_str(), Some("1.0.0"));
        }
        Ok(())
    }


    #[traced_test]
    async fn test_bump_with_multiple_downstreams() -> Result<(), CrateError> {
        let crate_a = CrateConfig::new("crate_a").with_src_files().with_readme();
        let crate_b = CrateConfig::new("crate_b").with_src_files().with_readme();
        let crate_c = CrateConfig::new("crate_c").with_src_files().with_readme();
        let tmp = create_mock_workspace(vec![crate_a, crate_b, crate_c])
            .await
            .expect("creation ok");

        // a->b, c->b
        for nm in ["crate_a", "crate_c"] {
            let cp = tmp.join(nm).join("Cargo.toml");
            let orig = fs::read_to_string(&cp).await.unwrap();
            let appended = format!(
                r#"{orig}
    [dependencies]
    crate_b = {{ path = "../crate_b", version = "0.1.0" }}
    "#
            );
            fs::write(&cp, appended).await.unwrap();
        }

        let mut ws = Workspace::<PathBuf, CrateHandle>::new(&tmp)
            .await
            .expect("workspace ok");

        let arc_b = ws.find_crate_by_name("crate_b").await
            .expect("crate_b not found");

        // Bump crate_b => Minor => "0.2.0"
        {
            let mut local_clone = {
                let gb = arc_b.lock().await;
                gb.clone()
            };
            ws.bump_crate_and_downstreams(&mut local_clone, ReleaseType::Minor)
                .await
                .expect("bump crate_b minor ok");
            {
                let mut gb = arc_b.lock().await;
                *gb = local_clone;
            }
        }

        // check crate_b => "0.2.0"
        {
            let path_b = {
                let gb = arc_b.lock().await;
                gb.as_ref().join("Cargo.toml")
            };
            let ver_b = read_crate_version(&path_b).await.unwrap();
            assert_eq!(ver_b, "0.2.0");
        }

        // check crate_a => references crate_b => "version=0.2.0"
        {
            let a_cp = tmp.join("crate_a").join("Cargo.toml");
            let doc_a = fs::read_to_string(&a_cp).await.unwrap().parse::<toml_edit::Document>().unwrap();
            let a_deps = doc_a["dependencies"].as_table().unwrap();
            let a_dep_b = &a_deps["crate_b"];
            let a_ver = a_dep_b.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(a_ver, "0.2.0");
        }

        // check crate_c => references crate_b => "version=0.2.0"
        {
            let c_cp = tmp.join("crate_c").join("Cargo.toml");
            let doc_c = fs::read_to_string(&c_cp).await.unwrap().parse::<toml_edit::Document>().unwrap();
            let c_deps = doc_c["dependencies"].as_table().unwrap();
            let c_dep_b = &c_deps["crate_b"];
            let c_ver = c_dep_b.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(c_ver, "0.2.0");
        }
        Ok(())
    }

    #[traced_test]
    async fn test_bump_with_chain_of_downstreams() -> Result<(), CrateError> {
        let crate_a = CrateConfig::new("crate_a").with_src_files().with_readme();
        let crate_b = CrateConfig::new("crate_b").with_src_files().with_readme();
        let crate_c = CrateConfig::new("crate_c").with_src_files().with_readme();
        let tmp = create_mock_workspace(vec![crate_a, crate_b, crate_c])
            .await
            .expect("workspace creation ok");

        // b->c, a->b
        let cargo_b_path = tmp.join("crate_b").join("Cargo.toml");
        let ob = fs::read_to_string(&cargo_b_path).await.unwrap();
        let ab = format!(
            r#"{ob}
    [dependencies]
    crate_c = {{ path = "../crate_c", version = "0.1.0" }}
    "#
        );
        fs::write(&cargo_b_path, ab).await.unwrap();

        let cargo_a_path = tmp.join("crate_a").join("Cargo.toml");
        let oa = fs::read_to_string(&cargo_a_path).await.unwrap();
        let aa = format!(
            r#"{oa}
    [dependencies]
    crate_b = {{ path = "../crate_b", version = "0.1.0" }}
    "#
        );
        fs::write(&cargo_a_path, aa).await.unwrap();

        let mut ws = Workspace::<PathBuf, CrateHandle>::new(&tmp)
            .await
            .expect("workspace ok");

        let arc_c = ws.find_crate_by_name("crate_c").await
            .expect("crate_c not found");

        // Bump crate_c => major => "1.0.0"
        {
            let mut local_clone = {
                let gc = arc_c.lock().await;
                gc.clone()
            };
            ws.bump_crate_and_downstreams(&mut local_clone, ReleaseType::Major)
                .await
                .expect("bump crate_c major ok");
            {
                let mut gc = arc_c.lock().await;
                *gc = local_clone;
            }
        }

        // check crate_c => 1.0.0
        {
            let path_c = {
                let g = arc_c.lock().await;
                g.as_ref().join("Cargo.toml")
            };
            let ver_c = read_crate_version(&path_c).await.unwrap();
            assert_eq!(ver_c, "1.0.0");
        }

        // check crate_b => references crate_c => "1.0.0"
        {
            let b_str = fs::read_to_string(&cargo_b_path).await.unwrap();
            let doc_b = b_str.parse::<toml_edit::Document>().unwrap();
            let b_deps = doc_b["dependencies"].as_table().unwrap();
            let b_dep_c = &b_deps["crate_c"];
            let b_c_ver = b_dep_c.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(b_c_ver, "1.0.0");
        }

        // check crate_a => references crate_b => "1.0.0"
        {
            let a_str = fs::read_to_string(&cargo_a_path).await.unwrap();
            let doc_a = a_str.parse::<toml_edit::Document>().unwrap();
            let a_deps = doc_a["dependencies"].as_table().unwrap();
            let a_dep_b = &a_deps["crate_b"];
            let a_b_ver = a_dep_b.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(a_b_ver, "1.0.0");
        }
        Ok(())
    }

    #[traced_test]
    async fn test_bump_crate_fails() -> Result<(), CrateError> {
        let crate_b = CrateConfig::new("b_fail").with_src_files().with_readme();
        let tmp = create_mock_workspace(vec![crate_b]).await.unwrap();

        let mut ws = Workspace::<PathBuf, CrateHandle>::new(&tmp).await.unwrap();
        let arc_b = ws.find_crate_by_name("b_fail").await
            .expect("b_fail crate not found");

        // sabotage: remove Cargo.toml
        {
            let path = {
                let gb = arc_b.lock().await;
                gb.as_ref().join("Cargo.toml")
            };
            fs::remove_file(&path).await.unwrap();
        }

        // Now attempt to bump => expect IoError (due to our new mapping)
        let mut local_b_clone = {
            let gb = arc_b.lock().await;
            gb.clone()
        };
        let res = ws.bump_crate_and_downstreams(&mut local_b_clone, ReleaseType::Patch).await;
        {
            let mut gb = arc_b.lock().await;
            *gb = local_b_clone;
        }

        match res {
            Err(WorkspaceError::BumpError { crate_path, source }) => {
                let crate_str = "b_fail/Cargo.toml".replace('/', &std::path::MAIN_SEPARATOR.to_string());
                assert!(
                    crate_path.ends_with(&crate_str),
                    "Expected failing path = b_fail’s cargo toml, got: {:?}",
                    crate_path
                );
                match *source {
                    // The test originally wants IoError
                    CrateError::IoError { .. } => {
                        // pass
                    }
                    other => panic!("Expected CrateError::IoError, got {:?}", other),
                }
            }
            other => panic!("Expected WorkspaceError::BumpError, got {:?}", other),
        }
        Ok(())
    }

    #[traced_test]
    async fn test_crate_handle_version_fails() -> Result<(), CrateError> {
        trace!("Starting test_crate_handle_version_fails");

        // 1) Create a normal "ver_fails" crate with a real Cargo.toml on disk
        //    plus a README. (We want the crate initially valid.)
        let single_cfg = CrateConfig::new("ver_fails").with_src_files().with_readme();
        let tmp2 = create_mock_workspace(vec![single_cfg])
            .await
            .expect("Failed to create mock workspace on disk");
        info!(
            "Created a mock workspace containing 'ver_fails' crate at {:?}",
            tmp2
        );

        let ws2 = Workspace::<PathBuf, CrateHandle>::new(&tmp2)
            .await
            .expect("Failed to create Workspace from disk");
        trace!("Workspace created successfully");

        // 2) Get our crate handle
        let arc_x2 = ws2
            .find_crate_by_name("ver_fails")
            .await
            .expect("ver_fails crate not found in workspace");
        info!("Located 'ver_fails' crate handle in the workspace");

        // 3) Bump normally => "0.1.1"
        {
            let mut local_clone = {
                let gx2 = arc_x2.lock().await;
                gx2.clone()
            };
            local_clone
                .bump(ReleaseType::Patch)
                .await
                .expect("Bump patch should succeed for a valid crate");

            info!("Bumped 'ver_fails' from 0.1.0 to 0.1.1 in-memory (and on disk)");

            let mut gx2 = arc_x2.lock().await;
            *gx2 = local_clone;
        }

        // 4) Sabotage the on-disk Cargo.toml => set final version = "not.semver"
        let cargo_x2 = {
            let gx2 = arc_x2.lock().await;
            gx2.as_ref().join("Cargo.toml")
        };
        info!("Sabotaging the Cargo.toml at {:?}", cargo_x2);

        let contents2 = fs::read_to_string(&cargo_x2).await.unwrap();
        let mut doc2 = contents2
            .parse::<toml_edit::Document>()
            .expect("parse cargo toml");
        if let Some(pkg_tbl) = doc2.get_mut("package").and_then(|val| val.as_table_mut()) {
            pkg_tbl.remove("version");
            pkg_tbl.insert("version", toml_edit::value("not.semver"));
        }
        fs::write(&cargo_x2, doc2.to_string())
            .await
            .expect("Failed to write sabotaged Cargo.toml");

        // Verify sabotage is actually written:
        let after_sabotage = fs::read_to_string(&cargo_x2).await.unwrap();
        assert!(
            after_sabotage.contains("not.semver"),
            "sabotage is not reflected in file! got: {after_sabotage}"
        );

        // >>>>>>> The one-line fix, forcing re-parse of cargo toml <<<<<<
        arc_x2.lock().await.validate_integrity().await.ok();  

        // 5) Now calling crate_handle.version() should yield an error
        info!(
            "Now calling crate_handle.version(), expecting an invalid-version error in {:?}",
            cargo_x2
        );
        {
            let handle_clone = {
                let gx2 = arc_x2.lock().await;
                gx2.clone()
            };
            let ret = handle_clone.version();
            match ret {
                Err(CrateError::CargoTomlError(
                    CargoTomlError::InvalidVersionFormat {
                        cargo_toml_file,
                        version,
                    },
                )) => {
                    // Confirm fields
                    assert_eq!(cargo_toml_file, cargo_x2);
                    assert_eq!(version, "not.semver".to_owned());
                    info!("Got the expected InvalidVersionFormat error for 'not.semver'");
                }
                Ok(_) => panic!("Expected parse error but got Ok"),
                other => panic!("Expected InvalidVersionFormat, got {:?}", other),
            }
        }

        info!("Finished test_crate_handle_version_fails successfully");
        Ok(())
    }
}
