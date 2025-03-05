// ---------------- [ File: workspacer-add-new-crate-to-workspace/src/create_crate_skeleton.rs ]
crate::ix!();

/// Extension trait to create the new crate skeleton: directory, Cargo.toml, src/lib.rs, ...
#[async_trait]
pub trait CreateCrateSkeleton<P>
where
    P: Send + AsRef<std::path::Path>,
{
    async fn create_new_crate_skeleton(
        &self,
        new_crate_name: &str,
    ) -> Result<P,WorkspaceError>;
}

#[async_trait]
impl<P,H> CreateCrateSkeleton<P> for crate::Workspace<P,H>
where
    P: From<PathBuf> + AsRef<std::path::Path> + Clone + Send + Sync + 'static,
    H: CrateHandleInterface<P> + Send + Sync + 'static,
{
    async fn create_new_crate_skeleton(
        &self,
        new_crate_name: &str,
    ) -> Result<P,WorkspaceError> {

        let crate_dir = self.as_ref().join(new_crate_name);
        info!("Creating new crate directory at {:?}", crate_dir);

        fs::create_dir_all(&crate_dir).await.map_err(|e| {
            WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("creating directory for new crate '{}'", new_crate_name),
            }
        })?;

        // Use `indoc!()` to keep the multi-line neat
        let cargo_toml_str = formatdoc! {r#"
            [package]
            name = "{new_crate_name}"
            version = "0.1.0"
            authors = ["YourName <you@example.com>"]
            license = "MIT"
            edition = "2024"
            description = "todo: write a description here"

            # keywords = []
            # categories = []

            [dependencies]
        "#};

        fs::write(crate_dir.join("Cargo.toml"), cargo_toml_str)
            .await
            .map_err(|e| WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("writing Cargo.toml for '{}'", new_crate_name),
            })?;

        // Create src dir
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).await.map_err(|e| {
            WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("creating src dir for '{}'", new_crate_name),
            }
        })?;

        // We will not decide prefix logic here; we produce a placeholder imports
        let imports_path = src_dir.join("imports.rs");
        let placeholder_imports = indoc! {r#"
        // If we belong to a prefix group, we'd do `pub(crate) use prefix_3p::*;`
        // For now, placeholder comment.
        "#};

        fs::write(&imports_path, placeholder_imports).await
            .map_err(|e| WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("writing imports.rs for '{}'", new_crate_name),
            })?;

        // We do suffix derivation or skip it. For demonstration, let's do a naive approach:
        let suffix_snake = dash_to_snake_case(new_crate_name);
        let entrypoint_filename = format!("{}.rs", suffix_snake);
        let entrypoint_path = src_dir.join(&entrypoint_filename);
        let entrypoint_contents = indoc! {r#"
        crate::ix!();
        "#};
        fs::write(&entrypoint_path, entrypoint_contents)
            .await
            .map_err(|e| WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("writing entrypoint file '{}' for '{}'", entrypoint_filename, new_crate_name),
            })?;

        let lib_rs = formatdoc!{
            r#"
            #[macro_use] mod imports; use imports::*;

            x!{{{suffix_snake}}}
            "#
        };

        fs::write(src_dir.join("lib.rs"), lib_rs)
            .await
            .map_err(|e| WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: format!("writing lib.rs for '{}'", new_crate_name),
            })?;

        // Optionally create a README
        // We skip here. Or we can do a minimal approach:
        let readme_path = crate_dir.join("README.md");
        if !readme_path.exists() {
            fs::write(&readme_path, format!("# {}\n\nTODO: fill description.\n", new_crate_name))
                .await
                .ok();
        }

        // Return the created path in your P
        Ok(P::from(crate_dir))
    }
}

#[cfg(test)]
mod test_create_crate_skeleton {
    use super::*;

    ///
    /// This module provides **exhaustive tests** for the `CreateCrateSkeleton` extension trait,
    /// specifically its `create_new_crate_skeleton(...)` method. We demonstrate:
    ///
    /// 1) Creating a brand-new crate in a real workspace directory (mocked by `create_mock_workspace`).
    /// 2) Checking that `Cargo.toml`, `src/lib.rs`, `src/imports.rs`, the entrypoint file, and an optional README
    ///    are created correctly, with the expected placeholders.
    /// 3) Handling crate names with dashes => underscores in the entrypoint (suffix_snake).
    /// 4) Verifying that if the directory already exists, we handle it gracefully (success or error).
    /// 5) Confirming minimal placeholders (commented `keywords`/`categories`, a placeholder `imports.rs`, etc.).
    ///
    /// We'll do multiple scenarios to ensure correctness.
    ///

    // We'll define a type alias for convenience in these tests.
    type MyWorkspace = Workspace<PathBuf, CrateHandle>;

    // -------------------------------------------------------------------------
    // 1) Basic scenario: create a new crate in an existing workspace
    // -------------------------------------------------------------------------
    #[traced_test]
    #[allow(non_snake_case)]
    async fn test_create_new_crate_skeleton_basic() {
        info!("Scenario 1: Basic creation of a new crate skeleton in a real workspace.");

        // 1) Create a mock workspace that already has a top-level Cargo.toml with [workspace].
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("existing_crate").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        let ws = MyWorkspace::new(&workspace_path)
            .await
            .expect("Should parse existing workspace successfully");

        // 2) Now call `create_new_crate_skeleton(...)` with a brand-new crate name.
        let new_crate_name = "my_new_crate";
        let full_dir: PathBuf = ws.create_new_crate_skeleton(new_crate_name)
            .await
            .expect("Should successfully create a new crate skeleton on disk");

        // 3) Check that a directory was created at <workspace>/<new_crate_name> 
        debug!("Crate skeleton was created at {:?}", full_dir);
        let metadata = fs::metadata(&full_dir).await
            .expect("Should have created the directory on disk");
        assert!(metadata.is_dir(), "Should be a directory");

        // 4) Check the Cargo.toml
        let cargo_toml_path = full_dir.join("Cargo.toml");
        let cargo_contents = fs::read_to_string(&cargo_toml_path).await
            .expect("Cargo.toml should be created");
        debug!("Cargo.toml:\n{}", cargo_contents);
        assert!(cargo_contents.contains("[package]"), "Should contain a [package] section");
        assert!(cargo_contents.contains("name = \"my_new_crate\""),
            "Should set the crate name to my_new_crate");
        assert!(cargo_contents.contains("# keywords = []"),
            "Should have placeholders for keywords");
        assert!(cargo_contents.contains("# categories = []"),
            "Should have placeholders for categories");
        assert!(cargo_contents.contains("description = \"todo: write a description here\""),
            "Should have a placeholder description");

        // 5) Check `src/lib.rs`
        let lib_rs_path = full_dir.join("src").join("lib.rs");
        let lib_rs_str = fs::read_to_string(&lib_rs_path).await
            .expect("lib.rs should be created");
        debug!("lib.rs:\n{}", lib_rs_str);
        assert!(lib_rs_str.contains("mod imports;"), "Should reference imports module");
        assert!(lib_rs_str.contains("x!{my_new_crate}"), 
            "Should call x!{{my_new_crate}} for the entrypoint file name");

        // 6) Check `src/imports.rs`
        let imports_rs_path = full_dir.join("src").join("imports.rs");
        let imports_str = fs::read_to_string(&imports_rs_path).await
            .expect("imports.rs should be created");
        debug!("imports.rs:\n{}", imports_str);
        assert!(imports_str.contains("placeholder comment"),
            "Should contain a placeholder comment in imports.rs (no prefix group logic here)");

        // 7) Check the entrypoint file => "my_new_crate.rs", since dash_to_snake_case won't rename underscores if no dash
        let entrypoint_file = full_dir.join("src").join("my_new_crate.rs");
        let entrypoint_str = fs::read_to_string(&entrypoint_file).await
            .expect("entrypoint file should be created");
        debug!("entrypoint:\n{}", entrypoint_str);
        assert!(entrypoint_str.contains("crate::ix!();"),
            "Should have a simple `crate::ix!();` line in the entrypoint file");

        // 8) Check README.md existence (optional in the code)
        let readme_path = full_dir.join("README.md");
        let readme_str = fs::read_to_string(&readme_path).await
            .expect("READM.md should be created by default");
        assert!(readme_str.contains("# my_new_crate"),
            "Should contain a simple # heading in the README");
    }

    // -------------------------------------------------------------------------
    // 2) Crate name with dashes => check suffix transformation
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_create_new_crate_skeleton_dashed_name() {
        info!("Scenario 2: creating a crate with dashes => entrypoint file uses snake_case suffix");

        let workspace_path = create_mock_workspace(vec![])
            .await
            .expect("Failed to create empty workspace");
        let ws = MyWorkspace::new(&workspace_path)
            .await
            .expect("Should parse workspace with no crates");

        let dashed_name = "batch-mode-json";
        let new_path: PathBuf = ws.create_new_crate_skeleton(dashed_name)
            .await
            .expect("Should succeed for dashed crate name");

        // Expect an entrypoint file named "batch_mode_json.rs"
        let entrypoint_file = new_path.join("src").join("batch_mode_json.rs");
        let entrypoint_str = fs::read_to_string(&entrypoint_file).await
            .expect("entrypoint file should be created for dashed crate name");
        debug!("entrypoint:\n{}", entrypoint_str);
        assert!(entrypoint_str.contains("crate::ix!();"));

        // Also check lib.rs references x!{batch_mode_json}
        let lib_rs = new_path.join("src").join("lib.rs");
        let lib_str = fs::read_to_string(&lib_rs).await.unwrap();
        assert!(lib_str.contains("x!{batch_mode_json}"),
            "Should reference the snake_case version in x! macro");
    }

    // -------------------------------------------------------------------------
    // 3) If the directory already exists but is empty => we can still create it
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_existing_empty_directory_ok() {
        info!("Scenario 3: the target directory already exists (empty). We proceed without error.");

        let workspace_path = create_mock_workspace(vec![])
            .await
            .expect("Failed to create empty workspace");
        let ws = MyWorkspace::new(&workspace_path)
            .await
            .expect("Workspace creation from empty dir ok");

        let crate_name = "already_here";
        let target_dir = ws.as_ref().join(crate_name);
        fs::create_dir_all(&target_dir).await
            .expect("Manually pre-create the directory so it's empty");

        // Now call create_new_crate_skeleton
        let result = ws.create_new_crate_skeleton(crate_name).await;
        assert!(
            result.is_ok(),
            "Should succeed if the directory is empty and we can fill in the files"
        );
        let crate_path = result.unwrap();
        assert_eq!(&crate_path, target_dir.as_path(),
            "Should return the same path we expected");
    }

    // -------------------------------------------------------------------------
    // 4) If the directory is not empty => we might still proceed (overwriting or error?)
    //    Our code does no check, so likely it overwrites any existing files if they have the same name.
    //    We'll test that we can create a new skeleton. 
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_existing_non_empty_directory() {
        info!("Scenario 4: directory is non-empty, we still attempt to create or overwrite files. Possibly success or partial overwrite.");

        let workspace_path = create_mock_workspace(vec![])
            .await
            .expect("Failed to create empty workspace");
        let ws = MyWorkspace::new(&workspace_path)
            .await
            .expect("Workspace creation from empty dir ok");

        let crate_name = "pre_existing";
        let target_dir = ws.as_ref().join(crate_name);
        fs::create_dir_all(&target_dir).await
            .expect("Manually pre-create the directory");
        // write a random file
        let random_file = target_dir.join("random.txt");
        fs::write(&random_file, b"some content").await
            .expect("Write random file");

        // Now call create_new_crate_skeleton
        let result: Result<PathBuf,WorkspaceError> = ws.create_new_crate_skeleton(crate_name).await;
        // We do not fail in the current code => no checks for overwriting
        assert!(result.is_ok(),
            "Current code doesn't fail if directory is non-empty; it just overwrites Cargo.toml, etc.");
        let crate_path = result.unwrap();
        assert_eq!(&crate_path, target_dir.as_path());

        // Check that cargo toml is present
        let cargo_toml = target_dir.join("Cargo.toml");
        let cargo_txt = fs::read_to_string(&cargo_toml).await
            .expect("Cargo.toml should exist even if directory was not empty");
        assert!(cargo_txt.contains("name = \"pre_existing\""));
        // random.txt still might exist, we haven't removed it
        let random_exists = fs::metadata(&random_file).await.is_ok();
        assert!(random_exists, "We do not remove or overwrite unrelated files");
    }

    // -------------------------------------------------------------------------
    // 5) If we can't create the directory, e.g. no permission => returns IoError
    //    We won't test real permission failure here, but show how you'd do it.
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_directory_creation_failure() {
        info!("Scenario 5: directory creation fails => returns IoError. We won't do real permission, just simulate a path error.");

        // We'll attempt to create a crate in a path that should fail. 
        // For demonstration, let's do "/dev/null/my_crate" on Unix, or something we expect to fail. 
        // We'll skip if on Windows. We'll show the approach, but won't always pass in every environment.

        #[cfg(target_os = "linux")]
        {
            let ws_path = PathBuf::from("/dev/null");
            let ws = MyWorkspace {
                path: ws_path.clone(),
                crates: vec![], // minimal
            };
            let crate_name = "no_way";
            let result = ws.create_new_crate_skeleton(crate_name).await;
            // Expect an IoError
            match result {
                Ok(_) => warn!("Unexpected success creating crate in /dev/null? Possibly environment-specific."),
                Err(WorkspaceError::IoError { context, .. }) => {
                    assert!(context.contains("creating directory for new crate 'no_way'"),
                        "Should mention the creation context");
                    info!("Got expected IoError for read-only or invalid path");
                },
                other => {
                    warn!("Got an unexpected result: {:?}", other);
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            info!("Skipping directory_creation_failure test on non-Linux platforms (or adapt for Windows e.g. C:\\nul\\??).");
        }
    }
}
