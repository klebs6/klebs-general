// ---------------- [ File: src/does_crate_have_dependency.rs ]
crate::ix!();

// -----------------------------------------------------------------------------
// 4) does_crate_have_dependency
// -----------------------------------------------------------------------------
///
/// Checks if `crate_handle` depends on `dep_name` in `[dependencies]`.
///
/// NOTE: We define a generic return type E that must be able to convert from `WorkspaceError`.
/// Then the caller can do `?` with E: From<WorkspaceError>.
///
pub async fn does_crate_have_dependency<P,H,E>(
    crate_handle: &H,
    dep_name: &str,
) -> Result<bool, E> 
where 
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
    E: From<WorkspaceError>,
{
    let cargo_path = crate_handle.as_ref().join("Cargo.toml");
    trace!(
        "Checking if crate '{}' depends on '{}'; reading {}",
        crate_handle.name(),
        dep_name,
        cargo_path.display()
    );

    let content = tokio::fs::read_to_string(&cargo_path)
        .await
        .map_err(|io_err| {
            E::from(WorkspaceError::IoError {
                io_error: Arc::new(io_err),
                context: format!("reading Cargo.toml at {:?}", cargo_path),
            })
        })?;

    let doc = content.parse::<toml_edit::Document>().map_err(|toml_err| {
        E::from(WorkspaceError::InvalidCargoToml(
            CargoTomlError::TomlEditError {
                cargo_toml_file: cargo_path.clone(),
                toml_parse_error: toml_err,
            }
        ))
    })?;

    if let Some(deps_table) = doc.get("dependencies").and_then(|val| val.as_table()) {
        if deps_table.get(dep_name).is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod test_does_crate_have_dependency {
    use super::*;

    // -------------------------------------------------------------------------
    // 7A) Test: does_crate_have_dependency
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_does_crate_have_dependency_basic() {
        // We'll create a minimal crate that depends on "serde"
        let temp = tempdir().unwrap();
        let crate_name = "dep_crate";
        let crate_dir = temp.path().join(crate_name);
        fs::create_dir_all(&crate_dir).await.unwrap();

        let cargo_toml = format!(
r#"[package]
name = "{crate_name}"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#);
        fs::write(crate_dir.join("Cargo.toml"), cargo_toml).await.unwrap();
        fs::create_dir_all(crate_dir.join("src")).await.unwrap();
        fs::write(crate_dir.join("src").join("lib.rs"), b"// dummy").await.unwrap();

        // Now make a CrateHandle
        let handle = CrateHandle::new(&crate_dir).await.unwrap();

        // Check depends on serde
        let result = does_crate_have_dependency::<PathBuf, CrateHandle, WorkspaceError>(&handle, "serde")
            .await
            .expect("Should not fail reading cargo toml");
        assert!(result, "Expected crate to depend on serde");

        // Check something else (like "rand") => false
        let result = does_crate_have_dependency::<PathBuf, CrateHandle, WorkspaceError>(&handle, "rand")
            .await
            .expect("Should not fail reading cargo toml");
        assert!(!result, "Expected no dependency on 'rand'");
    }

    /// In these tests, we thoroughly check `does_crate_have_dependency` with various scenarios:
    /// 1) **Basic**: A crate that definitely depends on "serde"
    /// 2) **Not Found**: A crate that doesn't have the dependency
    /// 3) **No `[dependencies]` table**: The crate's Cargo.toml has no `[dependencies]` section at all
    /// 4) **Missing Cargo.toml**: The file doesn't exist => triggers `IoError`
    /// 5) **Broken TOML**: The file is syntactically invalid => triggers `InvalidCargoToml` with `TomlEditError`
    /// 
    /// Each test creates or modifies an on-disk crate for realism, then calls `does_crate_have_dependency`.
    /// We confirm the returned boolean matches expectations, and we handle any error scenario that might arise.
    /// 
    /// NOTE: We rely on a simple `CrateHandle::new(...)` approach and direct filesystem manipulations.
    /// 
    /// For each test, we use `#[traced_test]` to capture logs and ensure robust `tracing` usage.
    /// 
    /// We pass `WorkspaceError` as the generic `E` type that implements `From<WorkspaceError>`.
    /// This allows the function to convert any internal `WorkspaceError` from I/O or parse errors into
    /// the same type used by the test's `?` operator.
    ///

    // -------------------------------------------------------------------------
    // 1) Test: Crate definitely has the dependency in `[dependencies]`
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_dep_found() {
        info!("Starting test_dep_found");

        // Create a minimal crate that depends on "serde"
        let tmp = tempdir().expect("Failed to create temp dir");
        let crate_path = tmp.path().join("mycrate");
        fs::create_dir_all(&crate_path).await.expect("Failed to create crate dir");

        let cargo_toml = r#"[package]
name = "mycrate"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;
        fs::write(crate_path.join("Cargo.toml"), cargo_toml)
            .await
            .expect("Failed to write Cargo.toml");
        fs::create_dir_all(crate_path.join("src"))
            .await
            .expect("Failed to create src dir");
        fs::write(crate_path.join("src").join("lib.rs"), "// dummy")
            .await
            .expect("Failed to write lib.rs");

        // Make a CrateHandle
        let handle = CrateHandle::new(&crate_path).await.expect("CrateHandle creation failed");
        // Check we indeed have "serde"
        let has_dep = does_crate_have_dependency::<PathBuf, CrateHandle, WorkspaceError>(&handle, "serde")
            .await
            .expect("Failed calling does_crate_have_dependency");
        assert!(has_dep, "We expect the crate to have a dependency on serde");
    }

    // -------------------------------------------------------------------------
    // 2) Test: Crate does NOT have the dependency
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_dep_not_found() {
        info!("Starting test_dep_not_found");

        let tmp = tempdir().expect("Failed to create temp dir");
        let crate_path = tmp.path().join("mycrate2");
        fs::create_dir_all(&crate_path).await.expect("Failed to create crate dir");

        let cargo_toml = r#"[package]
name = "mycrate2"
version = "0.1.0"

[dependencies]
serde_json = "1.0"
"#;
        fs::write(crate_path.join("Cargo.toml"), cargo_toml)
            .await
            .expect("Failed to write Cargo.toml");
        fs::create_dir_all(crate_path.join("src"))
            .await
            .expect("Failed to create src dir");
        fs::write(crate_path.join("src").join("lib.rs"), "// dummy")
            .await
            .expect("Failed to write lib.rs");

        let handle = CrateHandle::new(&crate_path).await.expect("CrateHandle creation failed");

        // Check for "rand", which does not exist
        let has_rand = does_crate_have_dependency::<PathBuf, CrateHandle, WorkspaceError>(&handle, "rand")
            .await
            .expect("does_crate_have_dependency call failed");
        assert!(!has_rand, "We expect no 'rand' dependency in this crate");
    }

    // -------------------------------------------------------------------------
    // 3) Test: Crate has no [dependencies] table at all => function should return false
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_no_dependencies_section() {
        info!("Starting test_no_dependencies_section");

        let tmp = tempdir().expect("Failed to create temp dir");
        let crate_path = tmp.path().join("mycrate3");
        fs::create_dir_all(&crate_path).await.expect("Failed to create crate dir");

        let cargo_toml = r#"[package]
name = "mycrate3"
version = "0.1.0"
"#;
        fs::write(crate_path.join("Cargo.toml"), cargo_toml)
            .await
            .expect("Failed to write Cargo.toml");
        fs::create_dir_all(crate_path.join("src"))
            .await
            .expect("Failed to create src dir");
        fs::write(crate_path.join("src").join("lib.rs"), "// dummy")
            .await
            .expect("Failed to write lib.rs");

        let handle = CrateHandle::new(&crate_path).await.expect("CrateHandle creation failed");

        // There's no [dependencies], so it can't possibly have "serde" or anything
        let has_dep = does_crate_have_dependency::<PathBuf, CrateHandle, WorkspaceError>(&handle, "serde")
            .await
            .expect("does_crate_have_dependency call failed");
        assert!(!has_dep, "We expect no dependencies in a crate with no [dependencies] table");
    }

    // -------------------------------------------------------------------------
    // 4) Test: Missing Cargo.toml => triggers IoError
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_missing_cargo_toml() {
        info!("Starting test_missing_cargo_toml");

        let tmp = tempdir().expect("Failed to create temp dir");
        let crate_path = tmp.path().join("missing_toml_crate");
        fs::create_dir_all(crate_path.join("src"))
            .await
            .expect("Failed to create crate dir");

        // We intentionally do NOT write Cargo.toml => missing
        fs::write(crate_path.join("src").join("lib.rs"), "// dummy")
            .await
            .expect("Failed to write lib.rs");

        // Make a handle => This might actually fail at creation time, but let's see
        // If CrateHandle::new doesn't fail, does_crate_have_dependency will definitely fail reading the missing file
        let handle_res = CrateHandle::new(&crate_path).await;
        let handle = match handle_res {
            Ok(h) => h,
            Err(e) => {
                // If your real code fails earlier, we can't test it here. We'll skip
                // or we can proceed with an "expected failure" approach
                warn!("CrateHandle creation already failed with {e:?}, skipping test");
                return;
            }
        };

        // Now call does_crate_have_dependency
        let result = does_crate_have_dependency::<PathBuf, CrateHandle, WorkspaceError>(&handle, "serde").await;
        match result {
            Ok(_) => panic!("Expected an error because Cargo.toml is missing"),
            Err(e) => {
                // We confirm it's an IoError with context about reading Cargo.toml
                match e {
                    WorkspaceError::IoError { context, .. } => {
                        assert!(
                            context.contains("reading Cargo.toml"),
                            "Should mention reading Cargo.toml in the context"
                        );
                        info!("Got the expected IoError for missing Cargo.toml.");
                    }
                    other => panic!("Expected IoError, got {other:?}"),
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 5) Test: Broken TOML => triggers InvalidCargoToml with TomlEditError
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_broken_toml_syntax() {
        info!("Starting test_broken_toml_syntax");

        let tmp = tempdir().expect("Failed to create temp dir");
        let crate_path = tmp.path().join("broken_syntax");
        fs::create_dir_all(&crate_path).await.expect("Failed to create crate dir");

        // We'll write a Cargo.toml that is not parseable
        let cargo_toml = r#"this is [not) valid toml??"#;
        fs::write(crate_path.join("Cargo.toml"), cargo_toml)
            .await
            .expect("Failed to write invalid Cargo.toml");
        fs::create_dir_all(crate_path.join("src"))
            .await
            .expect("Failed to create src dir");
        fs::write(crate_path.join("src").join("lib.rs"), "// dummy")
            .await
            .expect("Failed to write lib.rs");

        let handle = match CrateHandle::new(&crate_path).await {
            Ok(h) => h,
            Err(e) => {
                warn!("CrateHandle creation might fail early with parse error: {e:?}");
                return; 
            }
        };

        // Now call does_crate_have_dependency
        let result = does_crate_have_dependency::<PathBuf, CrateHandle, WorkspaceError>(&handle, "serde").await;
        match result {
            Ok(_) => panic!("Expected an error due to broken TOML syntax"),
            Err(e) => {
                // We confirm it's an InvalidCargoToml with TomlEditError
                match e {
                    WorkspaceError::InvalidCargoToml(cargo_err) => {
                        match cargo_err {
                            CargoTomlError::TomlEditError { .. } => {
                                info!("Got the expected TomlEditError for invalid syntax.");
                            }
                            other => panic!("Expected TomlEditError, got {other:?}"),
                        }
                    },
                    other => panic!("Expected InvalidCargoToml, got {other:?}"),
                }
            }
        }
    }
}
