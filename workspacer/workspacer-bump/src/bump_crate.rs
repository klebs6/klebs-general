// ---------------- [ File: workspacer-bump/src/bump_crate.rs ]
crate::ix!();

#[async_trait]
impl Bump for CrateHandle {
    type Error = CrateError;

    async fn bump(&mut self, release: ReleaseType) -> Result<(), Self::Error> {
        // Step 1: Lock + do synchronous changes
        let (cargo_toml_path, new_version_str) = {
            // Grab the path in a synchronous manner
            let cargo_path = self.cargo_toml_path_buf_sync()?;

            // Now lock the CargoToml for synchronous editing
            let cargo_toml_arc = self.cargo_toml();
            let mut guard = cargo_toml_arc.lock().await;

            // Validate
            guard.validate_integrity().await?;

            // Get + bump old version
            let mut old_ver = guard.version()?;
            release.apply_to(&mut old_ver);
            let new_ver_str = old_ver.to_string();

            // Overwrite in-memory
            {
                let pkg = guard.get_package_section_mut()?;
                pkg["version"] = toml::Value::String(new_ver_str.clone());
            }

            // Now we have the path and the new version string
            (cargo_path, new_ver_str)
            // guard is dropped here
        };

        // Step 2: Do the async write outside the guard
        {
            let cargo_toml_arc = self.cargo_toml();
            let mut guard = cargo_toml_arc.lock().await;
            guard.save_to_disk().await.map_err(|e| {
                // if needed, map error into CrateError
                // e.g. CrateError::IoError or similar
                e
            })?;

            tracing::info!(
                "Successfully bumped crate at {:?} to {}",
                cargo_toml_path,
                new_version_str
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_bump_crate_handle {
    use super::*;

    async fn read_package_version(cargo_toml_path: &Path) -> Option<String> {
        let contents = fs::read_to_string(cargo_toml_path).await.ok()?;
        let doc = contents.parse::<toml_edit::Document>().ok()?;
        let pkg = doc.get("package")?.as_table()?;
        let ver_item = pkg.get("version")?;
        ver_item.as_str().map(|s| s.to_string())
    }

    async fn setup_single_crate_handle(crate_name: &str) -> (tempfile::TempDir, Arc<AsyncMutex<CrateHandle>>) {
        let single_cfg = CrateConfig::new(crate_name).with_src_files();
        let root_path = create_mock_workspace(vec![single_cfg])
            .await
            .expect("Failed to create mock workspace");

        let raw_handle = CrateHandle::new(&root_path.join(crate_name))
            .await
            .expect("Failed to create CrateHandle");

        let arc_handle = Arc::new(AsyncMutex::new(raw_handle));
        (tempfile::TempDir::new_in(root_path.parent().unwrap()).unwrap(), arc_handle)
    }

    // Remove #[traced_test], use plain #[tokio::test] or just no attribute:
    #[tokio::test]
    async fn test_bump_major_ok() {
        let (_temp, arc_handle) = setup_single_crate_handle("major_ok").await;

        // Check initial
        {
            let path = {
                let g = arc_handle.lock().await;
                g.as_ref().join("Cargo.toml")
            };
            let initial_ver = read_package_version(&path).await.expect("initial version");
            assert_eq!(initial_ver, "0.1.0");
        }

        // Bump => Major
        {
            let mut local_clone = {
                let g = arc_handle.lock().await;
                g.clone()
            };
            local_clone.bump(ReleaseType::Major).await.expect("bump major ok");
            // store it back
            {
                let mut g = arc_handle.lock().await;
                *g = local_clone;
            }
        }

        // Confirm => 1.0.0
        {
            let path = {
                let g = arc_handle.lock().await;
                g.as_ref().join("Cargo.toml")
            };
            let updated_ver = read_package_version(&path).await.expect("updated version");
            assert_eq!(updated_ver, "1.0.0");
        }
    }

    #[tokio::test]
    async fn test_bump_minor_ok() {
        let (_temp, arc_handle) = setup_single_crate_handle("minor_ok").await;
        {
            let mut local_clone = {
                let g = arc_handle.lock().await;
                g.clone()
            };
            local_clone.bump(ReleaseType::Minor).await.expect("bump minor ok");
            {
                let mut g = arc_handle.lock().await;
                *g = local_clone;
            }
        }
        // verify
        {
            let path = {
                let g = arc_handle.lock().await;
                g.as_ref().join("Cargo.toml")
            };
            let ver = read_package_version(&path).await.unwrap();
            assert_eq!(ver, "0.2.0");
        }
    }

    #[tokio::test]
    async fn test_bump_patch_ok() {
        let (_temp, arc_handle) = setup_single_crate_handle("patch_ok").await;
        {
            let mut local_clone = {
                let g = arc_handle.lock().await;
                g.clone()
            };
            local_clone.bump(ReleaseType::Patch).await.expect("bump patch ok");
            {
                let mut g = arc_handle.lock().await;
                *g = local_clone;
            }
        }
        // read final
        {
            let p = {
                let gd = arc_handle.lock().await;
                gd.as_ref().join("Cargo.toml")
            };
            let v = read_package_version(&p).await.unwrap();
            assert_eq!(v, "0.1.1");
        }
    }

    #[tokio::test]
    async fn test_bump_alpha_ok() {
        let (_temp, arc_handle) = setup_single_crate_handle("alpha_ok").await;

        // alpha(None)
        {
            let mut lc = {
                let g = arc_handle.lock().await;
                g.clone()
            };
            lc.bump(ReleaseType::Alpha(None)).await.expect("bump alpha none");
            {
                let mut g = arc_handle.lock().await;
                *g = lc;
            }
        }
        {
            let path = {
                let gd = arc_handle.lock().await;
                gd.as_ref().join("Cargo.toml")
            };
            let alpha_ver = read_package_version(&path).await.unwrap();
            assert_eq!(alpha_ver, "0.1.0-alpha1");
        }

        // alpha(Some(99))
        {
            let mut lc = {
                let g = arc_handle.lock().await;
                g.clone()
            };
            lc.bump(ReleaseType::Alpha(Some(99))).await.expect("bump alpha 99");
            {
                let mut g = arc_handle.lock().await;
                *g = lc;
            }
        }
        {
            let path = {
                let gd = arc_handle.lock().await;
                gd.as_ref().join("Cargo.toml")
            };
            let alpha99 = read_package_version(&path).await.unwrap();
            assert_eq!(alpha99, "0.1.0-alpha99");
        }
    }

    #[tokio::test]
    async fn test_missing_package_section_error() {
        let (_temp, arc_handle) = setup_single_crate_handle("missing_package").await;
        // sabotage
        {
            let p = {
                let g = arc_handle.lock().await;
                g.as_ref().join("Cargo.toml")
            };
            let contents = fs::read_to_string(&p).await.unwrap();
            let sabotage = contents.replace("[package]", "#[package]");
            fs::write(&p, sabotage).await.unwrap();
        }

        let mut local_clone = {
            let g = arc_handle.lock().await;
            g.clone()
        };
        let result = local_clone.bump(ReleaseType::Patch).await;
        {
            let mut gg = arc_handle.lock().await;
            *gg = local_clone;
        }
        match result {
            Err(CrateError::CargoTomlError(
                CargoTomlError::MissingPackageSection { cargo_toml_file }
            )) => {
                let real_path = {
                    let gg = arc_handle.lock().await;
                    gg.as_ref().join("Cargo.toml")
                };
                assert_eq!(cargo_toml_file, real_path);
            }
            other => panic!("Expected MissingPackageSection, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_missing_version_key_error() {
        let (_temp, arc_handle) = setup_single_crate_handle("missing_version_key").await;
        // sabotage
        {
            let p = {
                let gd = arc_handle.lock().await;
                gd.as_ref().join("Cargo.toml")
            };
            let s = fs::read_to_string(&p).await.unwrap();
            let sabotage = s.replace("version = \"0.1.0\"", "");
            fs::write(&p, sabotage).await.unwrap();
        }
        let mut lc = {
            let g = arc_handle.lock().await;
            g.clone()
        };
        let result = lc.bump(ReleaseType::Major).await;
        {
            let mut g = arc_handle.lock().await;
            *g = lc;
        }
        match result {
            Err(CrateError::CargoTomlError(
                CargoTomlError::MissingRequiredFieldForIntegrity { cargo_toml_file, field }
            )) => {
                let real_path = {
                    let gg = arc_handle.lock().await;
                    gg.as_ref().join("Cargo.toml")
                };
                assert_eq!(cargo_toml_file, real_path);
                assert_eq!(field, "version");
            }
            other => panic!("Expected MissingRequiredFieldForIntegrity, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_invalid_version_format() {
        let (_temp, arc_handle) = setup_single_crate_handle("invalid_version_format").await;
        {
            let path = {
                let g = arc_handle.lock().await;
                g.as_ref().join("Cargo.toml")
            };
            let c = fs::read_to_string(&path).await.unwrap();
            let sabotage = c.replace("version = \"0.1.0\"", "version = \"not.semver\"");
            fs::write(&path, sabotage).await.unwrap();
        }
        let mut lc = {
            let g = arc_handle.lock().await;
            g.clone()
        };
        let result = lc.bump(ReleaseType::Patch).await;
        {
            let mut g = arc_handle.lock().await;
            *g = lc;
        }
        match result {
            Err(CrateError::CargoTomlError(
                CargoTomlError::InvalidVersionFormat { cargo_toml_file, version }
            )) => {
                let real_path = {
                    let gg = arc_handle.lock().await;
                    gg.as_ref().join("Cargo.toml")
                };
                assert_eq!(cargo_toml_file, real_path);
                assert_eq!(version, "not.semver");
            }
            other => panic!("Expected InvalidVersionFormat, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_io_error_missing_cargo_toml() {
        let (_temp, arc_handle) = setup_single_crate_handle("io_error_missing_toml").await;
        {
            let path = {
                let g = arc_handle.lock().await;
                g.as_ref().join("Cargo.toml")
            };
            fs::remove_file(&path).await.unwrap();
        }
        let mut lc = {
            let g = arc_handle.lock().await;
            g.clone()
        };
        let result = lc.bump(ReleaseType::Patch).await;
        {
            let mut g = arc_handle.lock().await;
            *g = lc;
        }
        match result {
            Err(CrateError::IoError { context, .. }) => {
                assert!(context.contains("reading"), "Should mention reading cargo toml");
            }
            other => panic!("Expected IoError from missing Cargo.toml, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_alpha_huge_number() {
        let (_temp, arc_handle) = setup_single_crate_handle("alpha_huge").await;
        let big_num = 999999999999999999u64;
        let mut lc = {
            let g = arc_handle.lock().await;
            g.clone()
        };
        let res = lc.bump(ReleaseType::Alpha(Some(big_num))).await;
        {
            let mut g = arc_handle.lock().await;
            *g = lc;
        }
        assert!(res.is_ok(), "Should not fail semver parse for large alpha number");

        let cargo_toml_path = {
            let gd = arc_handle.lock().await;
            gd.as_ref().join("Cargo.toml")
        };
        let new_ver_str = read_package_version(&cargo_toml_path).await.expect("expected version");
        assert!(
            new_ver_str.contains("alpha"),
            "Expected alpha in the version, got: {new_ver_str}"
        );
    }
}
