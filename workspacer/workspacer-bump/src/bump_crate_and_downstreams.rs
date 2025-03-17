// ---------------- [ File: workspacer-bump/src/bump_crate_and_downstreams.rs ]
crate::ix!();

#[async_trait]
impl<P,H> BumpCrateAndDownstreams for Workspace<P,H>
where
    // No 'static bound on H
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Bump<Error=CrateError> + Send + Sync,
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

        // 1) Bump this crate
        crate_handle.bump(release.clone()).await.map_err(|err| {
            error!("Error bumping crate '{}': {:?}", crate_handle.name(), err);
            WorkspaceError::BumpError {
                crate_path: crate_handle.as_ref().to_path_buf(),
                source: Box::new(err),
            }
        })?;

        // 2) Read the new version
        let new_ver = crate_handle.version().map_err(|ver_err| {
            error!("Could not re-parse new version for '{}': {:?}", crate_handle.name(), ver_err);
            WorkspaceError::CrateError(ver_err)
        })?;

        info!(
            "bump_crate_and_downstreams: crate='{}' => new_ver={}",
            crate_handle.name(),
            new_ver
        );

        // 3) Recursively update all downstream crates
        let crate_key = crate_handle.name().to_string();
        let mut visited = HashSet::new();
        visited.insert(crate_key.clone());
        self.update_downstreams_recursively(&crate_key, &new_ver, &mut visited).await?;

        Ok(())
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
    ) -> (TempDir, Workspace<PathBuf, CrateHandle>, Arc<Mutex<CrateHandle>>) {
        let tmp_root = create_mock_workspace(crate_configs)
            .await
            .expect("Failed to create mock workspace");

        let workspace = Workspace::<PathBuf, CrateHandle>::new(&tmp_root)
            .await
            .expect("Failed to create Workspace");

        let maybe_ch = workspace.find_crate_by_name(target_crate_name);
        let ch = maybe_ch.expect(&format!("Cannot find crate '{}'", target_crate_name));

        (
            TempDir::new_in(tmp_root.parent().unwrap()).unwrap(),
            workspace,
            ch
        )
    }

    // NOTE: We remove #[traced_test] so it's just a normal async test now:
    #[tokio::test]
    async fn test_bump_with_no_downstreams() -> Result<(), CrateError> {
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let (_temp, mut workspace, arc_a) =
            setup_workspace_and_handle(vec![crate_a, crate_b], "crate_a").await;

        // Confirm initial version
        {
            let handle_clone = {
                let g = arc_a.lock().expect("lock handle_a");
                g.clone()
            };
            assert_eq!(handle_clone.version()?.to_string(), "0.1.0");
        }

        // Bump crate_a => patch => "0.1.1"
        {
            let mut local_clone = {
                let g = arc_a.lock().expect("lock handle_a");
                g.clone()
            };
            workspace
                .bump_crate_and_downstreams(&mut local_clone, ReleaseType::Patch)
                .await
                .expect("bump crate_a patch ok");
            // store it back
            {
                let mut g = arc_a.lock().expect("re-lock handle_a");
                *g = local_clone;
            }
        }

        // Confirm crate_a => 0.1.1
        {
            let path_a = {
                let g = arc_a.lock().expect("lock handle_a");
                g.as_ref().join("Cargo.toml")
            };
            let new_ver_a = read_crate_version(&path_a).await.unwrap();
            assert_eq!(new_ver_a, "0.1.1");
        }

        // Confirm crate_b => still "0.1.0"
        {
            let arc_b = workspace.find_crate_by_name("crate_b").unwrap();
            let path_b = {
                let gb = arc_b.lock().expect("lock handle_b");
                gb.as_ref().join("Cargo.toml")
            };
            let ver_b = read_crate_version(&path_b).await.unwrap();
            assert_eq!(ver_b, "0.1.0", "crate_b should be unchanged");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_bump_with_single_downstream() -> Result<(), CrateError> {
        let crate_a_cfg = CrateConfig::new("crate_a").with_src_files();
        let crate_b_cfg = CrateConfig::new("crate_b").with_src_files();
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
        let arc_b = ws.find_crate_by_name("crate_b").expect("crate_b not found");

        // Bump crate_b => major => "1.0.0"
        {
            let mut local_clone = {
                let g = arc_b.lock().expect("lock handle_b");
                g.clone()
            };
            ws.bump_crate_and_downstreams(&mut local_clone, ReleaseType::Major)
                .await
                .expect("bump crate_b major ok");
            {
                let mut g = arc_b.lock().expect("store back b");
                *g = local_clone;
            }
        }

        // Check crate_b => "1.0.0"
        {
            let path_b = {
                let gb = arc_b.lock().expect("lock handle_b");
                gb.as_ref().join("Cargo.toml")
            };
            let ver_b = read_crate_version(&path_b).await.unwrap();
            assert_eq!(ver_b, "1.0.0");
        }

        // Check crate_a => references crate_b => "version=1.0.0"
        {
            let new_a_contents = fs::read_to_string(&a_cargo_path).await.unwrap();
            let doc_a = new_a_contents.parse::<TomlEditDocument>().unwrap();
            let deps_table = doc_a["dependencies"].as_table().unwrap();
            let crate_b_dep = &deps_table["crate_b"];
            assert!(crate_b_dep.is_table());
            let ver_item = crate_b_dep.as_table().unwrap().get("version").unwrap();
            assert_eq!(ver_item.as_str(), Some("1.0.0"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_bump_with_multiple_downstreams() -> Result<(), CrateError> {
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let crate_c = CrateConfig::new("crate_c").with_src_files();
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
        let arc_b = ws.find_crate_by_name("crate_b").expect("crate_b missing");

        // Bump crate_b => Minor => "0.2.0"
        {
            let mut local_clone = {
                let gb = arc_b.lock().expect("lock b");
                gb.clone()
            };
            ws.bump_crate_and_downstreams(&mut local_clone, ReleaseType::Minor)
                .await
                .expect("bump crate_b minor ok");
            {
                let mut gb = arc_b.lock().expect("store b back");
                *gb = local_clone;
            }
        }

        // check crate_b => "0.2.0"
        {
            let path_b = {
                let gb = arc_b.lock().expect("lock b final");
                gb.as_ref().join("Cargo.toml")
            };
            let ver_b = read_crate_version(&path_b).await.unwrap();
            assert_eq!(ver_b, "0.2.0");
        }

        // check crate_a => references crate_b => "version=0.2.0"
        {
            let a_cp = tmp.join("crate_a").join("Cargo.toml");
            let doc_a = fs::read_to_string(&a_cp).await.unwrap().parse::<TomlEditDocument>().unwrap();
            let a_deps = doc_a["dependencies"].as_table().unwrap();
            let a_dep_b = &a_deps["crate_b"];
            let a_ver = a_dep_b.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(a_ver, "0.2.0");
        }

        // check crate_c => references crate_b => "version=0.2.0"
        {
            let c_cp = tmp.join("crate_c").join("Cargo.toml");
            let doc_c = fs::read_to_string(&c_cp).await.unwrap().parse::<TomlEditDocument>().unwrap();
            let c_deps = doc_c["dependencies"].as_table().unwrap();
            let c_dep_b = &c_deps["crate_b"];
            let c_ver = c_dep_b.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(c_ver, "0.2.0");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_bump_with_chain_of_downstreams() -> Result<(), CrateError> {
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let crate_c = CrateConfig::new("crate_c").with_src_files();
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
        let arc_c = ws.find_crate_by_name("crate_c").expect("crate_c missing");

        // Bump crate_c => major => "1.0.0"
        {
            let mut local_clone = {
                let gc = arc_c.lock().expect("lock c");
                gc.clone()
            };
            ws.bump_crate_and_downstreams(&mut local_clone, ReleaseType::Major)
                .await
                .expect("bump crate_c major ok");
            {
                let mut gc = arc_c.lock().expect("store c back");
                *gc = local_clone;
            }
        }

        // check crate_c => 1.0.0
        {
            let path_c = {
                let g = arc_c.lock().expect("lock c");
                g.as_ref().join("Cargo.toml")
            };
            let ver_c = read_crate_version(&path_c).await.unwrap();
            assert_eq!(ver_c, "1.0.0");
        }

        // check crate_b => references crate_c => "1.0.0"
        {
            let b_str = fs::read_to_string(&cargo_b_path).await.unwrap();
            let doc_b = b_str.parse::<TomlEditDocument>().unwrap();
            let b_deps = doc_b["dependencies"].as_table().unwrap();
            let b_dep_c = &b_deps["crate_c"];
            let b_c_ver = b_dep_c.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(b_c_ver, "1.0.0");
        }

        // check crate_a => references crate_b => "1.0.0"
        {
            let a_str = fs::read_to_string(&cargo_a_path).await.unwrap();
            let doc_a = a_str.parse::<TomlEditDocument>().unwrap();
            let a_deps = doc_a["dependencies"].as_table().unwrap();
            let a_dep_b = &a_deps["crate_b"];
            let a_b_ver = a_dep_b.as_table().unwrap()["version"].as_str().unwrap();
            assert_eq!(a_b_ver, "1.0.0");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_bump_crate_fails() -> Result<(), CrateError> {
        let crate_b = CrateConfig::new("b_fail").with_src_files();
        let tmp = create_mock_workspace(vec![crate_b]).await.unwrap();

        let mut ws = Workspace::<PathBuf, CrateHandle>::new(&tmp).await.unwrap();
        let arc_b = ws.find_crate_by_name("b_fail").unwrap();

        // sabotage
        {
            let path_b = {
                let gb = arc_b.lock().expect("lock crate_b");
                gb.as_ref().join("Cargo.toml")
            };
            fs::remove_file(&path_b).await.unwrap();
        }

        // Now attempt to bump => expect IoError
        let mut local_b_clone = {
            let gb = arc_b.lock().expect("lock handle_b");
            gb.clone()
        };
        let res = ws.bump_crate_and_downstreams(&mut local_b_clone, ReleaseType::Patch).await;
        {
            let mut gb = arc_b.lock().expect("store b back anyway");
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
                    CrateError::IoError { ref context, .. } => {
                        assert!(
                            context.contains("reading"),
                            "Should mention reading cargo toml in context: got {context}"
                        );
                    }
                    other => panic!("Expected CrateError::IoError, got {:?}", other),
                }
            }
            other => panic!("Expected WorkspaceError::BumpError, got {:?}", other),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_crate_handle_version_fails() -> Result<(), CrateError> {
        let single_cfg = CrateConfig::new("ver_fails").with_src_files();
        let tmp2 = create_mock_workspace(vec![single_cfg]).await.unwrap();
        let mut ws2 = Workspace::<PathBuf, CrateHandle>::new(&tmp2).await.unwrap();
        let arc_x2 = ws2.find_crate_by_name("ver_fails").unwrap();

        // normal bump => "0.1.0" => "0.1.1"
        {
            let mut local_clone = {
                let gx2 = arc_x2.lock().expect("lock handle_x2");
                gx2.clone()
            };
            local_clone.bump(ReleaseType::Patch).await.expect("bump patch ok");
            {
                let mut gx2 = arc_x2.lock().expect("store x2");
                *gx2 = local_clone;
            }
        }

        // sabotage => set final version => "not.semver"
        let cargo_x2 = {
            let gx2 = arc_x2.lock().expect("lock handle_x2 read path");
            gx2.as_ref().join("Cargo.toml")
        };
        let contents2 = fs::read_to_string(&cargo_x2).await.unwrap();
        let sabotage2 = contents2.replace("0.1.1", "not.semver");
        fs::write(&cargo_x2, sabotage2).await.unwrap();

        // now crate_handle.version() should fail
        let handle_clone = {
            let gx2 = arc_x2.lock().expect("lock handle_x2 final");
            gx2.clone()
        };
        let ret = handle_clone.version();
        match ret {
            Err(CrateError::CargoTomlError(
                CargoTomlError::InvalidVersionFormat { cargo_toml_file, version }
            )) => {
                assert_eq!(cargo_toml_file, cargo_x2);
                assert_eq!(version, "not.semver".to_string());
            }
            Ok(_) => panic!("Expected parse error but got Ok"),
            other => panic!("Expected InvalidVersionFormat, got {:?}", other),
        }
        Ok(())
    }
}
