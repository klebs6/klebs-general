// ---------------- [ File: workspacer-bump/src/workspace_bump_all.rs ]
crate::ix!();

/// Macro that generates an `impl BumpAll for SomeWorkspace<P,T>` block.
/// Use it once for your real `Workspace<P,T>` and once for your `MockWorkspace<P,T>`.
#[macro_export]
macro_rules! gen_bump_all_for_workspace {
    ($ws_type:ident) => {
        // We bring all the needed items into scope, 
        // but if you're defining the macro in the same module, 
        // you may not need as many `use` lines.

        #[async_trait]
        impl<P, T> BumpAll for $ws_type<P, T>
        where
            // The workspace must implement your `WorkspaceInterface<P,T>`
            // so we can call self.crates().
            Self: WorkspaceInterface<P, T> + Send + Sync,

            // P must be from PathBuf, etc.
            for<'any> P: From<std::path::PathBuf> 
                + AsRef<std::path::Path> 
                + Send 
                + Sync 
                + Clone
                + 'any,

            // T must be your crate handle, which also implements Bump
            for<'any> T: CrateHandleInterface<P> 
                + Bump<Error = CrateError> 
                + Send 
                + Sync 
                + Clone
                + 'any,
        {
            type Error = WorkspaceError;

            async fn bump_all(&mut self, release: ReleaseType) 
                -> Result<(), Self::Error> 
            {
                tracing::trace!(
                    "Entering blanket-impl bump_all for {} with release={:?}",
                    stringify!($ws_type),
                    release
                );

                let mut updated_versions = std::collections::HashMap::<String, String>::new();

                // --- First pass: bump each crate
                for arc_crate in self.crates() {
                    let mut guard = arc_crate.lock().await;
                    let crate_name: String = guard.name().to_string(); 
                    // (Fix Cow<str> -> String)

                    match guard.bump(release.clone()).await {
                        Ok(()) => {
                            // retrieve new version
                            match guard.version() {
                                Ok(ver) => {
                                    updated_versions.insert(crate_name, ver.to_string());
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Could not parse new version for '{}': {:?}",
                                        crate_name, 
                                        e
                                    );
                                    return Err(
                                        WorkspaceError::BumpError {
                                            crate_path: guard.as_ref().join("Cargo.toml"),
                                            source: Box::new(e),
                                        }
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Crate '{}' failed to bump => returning an error",
                                crate_name
                            );
                            return Err(
                                WorkspaceError::BumpError {
                                    crate_path: guard.as_ref().join("Cargo.toml"),
                                    source: Box::new(e),
                                }
                            );
                        }
                    }
                }

                tracing::info!(
                    "All crates in {} bumped successfully. Now doing 2nd pass updates.",
                    stringify!($ws_type)
                );

                // --- Second pass: rewrite dependencies
                for arc_crate in self.crates() {
                    let handle = arc_crate.lock().await;
                    let crate_name = handle.name().to_string();

                    let cargo_toml_arc = handle.cargo_toml();
                    let mut cargo_toml = cargo_toml_arc.lock().await;

                    let mut changed_any = false;
                    for (dep_name, new_ver) in &updated_versions {
                        match cargo_toml.update_dependency_version(dep_name, new_ver) {
                            Ok(true) => {
                                changed_any = true;
                                tracing::trace!(
                                    "[2nd pass] crate='{}': updated dep='{}' => '{}'",
                                    crate_name, 
                                    dep_name, 
                                    new_ver
                                );
                            }
                            Ok(false) => {
                                // not referenced
                            }
                            Err(e) => {
                                // log but do not short-circuit entire workspace
                                tracing::error!(
                                    "Failed updating '{}' in '{}': {:?}",
                                    dep_name,
                                    crate_name,
                                    e
                                );
                            }
                        }
                    }

                    if changed_any {
                        if let Err(save_err) = cargo_toml.save_to_disk().await {
                            tracing::error!(
                                "Could not rewrite references in '{}' Cargo.toml: {:?}",
                                crate_name, 
                                save_err
                            );
                        }
                    }
                }

                tracing::info!(
                    "Finished bump_all successfully for {} with release={:?}.",
                    stringify!($ws_type),
                    release
                );
                Ok(())
            }
        }
    };
}

gen_bump_all_for_workspace!(Workspace);

#[cfg(test)]
gen_bump_all_for_workspace!(MockWorkspace);

#[cfg(test)]
mod test_mock_workspace_bumping {

    use super::*;

    /// A helper to extract the "version" string from a given Cargo.toml path.
    async fn read_crate_version(cargo_toml_path: &std::path::Path) -> Option<String> {
        trace!("read_crate_version: about to read {cargo_toml_path:?}");
        let contents = fs::read_to_string(cargo_toml_path).await.ok()?;
        let doc = contents.parse::<TomlEditDocument>().ok()?;
        let pkg = doc.get("package")?.as_table()?;
        let ver_item = pkg.get("version")?;
        ver_item.as_str().map(|s| s.to_string())
    }

    #[traced_test]
    async fn test_bump_with_no_downstreams() -> Result<(), CrateError> {
        trace!("test_bump_with_no_downstreams => starting");

        // Create actual in-memory mock directories/crates on disk to test (like an integration style):
        let crate_a = CrateConfig::new("crate_a").with_src_files().with_readme();
        let crate_b = CrateConfig::new("crate_b").with_src_files().with_readme();

        // This helper is from `create_mock_workspace`, which is also local in crate::mock:
        let tmp_root = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("Failed to create mock workspace on disk");

        let mut workspace = Workspace::<PathBuf, CrateHandle>::new(&tmp_root)
            .await
            .expect("Failed to create real-lifish Workspace from mock fs data");

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
    async fn test_mock_workspace_bump_all_patch() {
        trace!("test_mock_workspace_bump_all_patch => starting");

        // 1) Create a couple of mock crates in memory (both are MockCrateHandles):
        let crate_a = Arc::new(AsyncMutex::new(
            MockCrateHandle::fully_valid_config()
                .to_builder()
                .crate_name("crate_a")
                .build()
                .unwrap()
        ));
        let crate_b = Arc::new(AsyncMutex::new(
            MockCrateHandle::fully_valid_config()
                .to_builder()
                .crate_name("crate_b")
                .build()
                .unwrap()
        ));

        // 2) Build a local mock workspace using <PathBuf, MockCrateHandle>,
        //    because we're passing Arc<AsyncMutex<MockCrateHandle>> items.
        let mut my_ws = MockWorkspaceBuilder::<PathBuf, MockCrateHandle>::default()
            .path(std::path::PathBuf::from("/fake/mock/workspace/path"))
            .crates(vec![crate_a.clone(), crate_b.clone()])
            .simulate_missing_cargo_toml(false)
            .simulate_not_a_workspace(false)
            .simulate_failed_integrity(false)
            .simulate_no_crates(false)
            .build()
            .unwrap();

        // 3) Bump all => patch => (1.2.3 -> 1.2.4)
        my_ws
            .bump_all(ReleaseType::Patch)
            .await
            .expect("Bumping all crates patch-version should succeed in mock workspace");

        // 4) Confirm crate_a => "1.2.4"
        {
            let arc_a = my_ws
                .find_crate_by_name("crate_a")
                .await
                .expect("Expected crate_a in the mock workspace");
            let ver_a = arc_a.lock().await.version().unwrap();
            assert_eq!(
                ver_a.to_string(),
                "1.2.4",
                "crate_a should be patched from 1.2.3 -> 1.2.4 in mock scenario"
            );
        }

        // 5) Confirm crate_b => "1.2.4"
        {
            let arc_b = my_ws
                .find_crate_by_name("crate_b")
                .await
                .expect("Expected crate_b in the mock workspace");
            let ver_b = arc_b.lock().await.version().unwrap();
            assert_eq!(
                ver_b.to_string(),
                "1.2.4",
                "crate_b should be patched from 1.2.3 -> 1.2.4 in memory"
            );
        }

        info!("test_mock_workspace_bump_all_patch => finished successfully");
    }

    #[traced_test]
    async fn test_mock_workspace_fails_integrity() {
        trace!("test_mock_workspace_fails_integrity => starting");

        // Build a "fully valid" mock workspace, then sabotage it:
        let mut broken_ws = MockWorkspace::<PathBuf,CrateHandle>::fully_valid_config();
        *broken_ws.simulate_missing_cargo_toml_mut() = true;

        // Now calling validate_integrity() should fail
        let result = broken_ws.validate_integrity().await;
        assert!(
            result.is_err(),
            "We set simulate_missing_cargo_toml=true => should fail"
        );

        info!("Got expected error result => {result:?}");
        info!("test_mock_workspace_fails_integrity => finished successfully");
    }
}
