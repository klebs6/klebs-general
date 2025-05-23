// ---------------- [ File: workspacer-add-internal-dep/src/add_internal_dep.rs ]
crate::ix!();

// ---------------------- [ File: workspacer-add-internal-dep/src/lib.rs ] ----------------------
/// A trait for adding an internal dep from one crate to another.
///
#[async_trait]
pub trait AddInternalDependency<P,H> 
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Debug + Send + Sync,
{
    type Error;
    /// Attempt to add a dependency on `dep_crate` to the `target_crate`.
    /// - Edits the target crate’s Cargo.toml to add `[dependencies] dep_name = { path=... }`
    /// - Updates `src/imports.rs` with a `pub(crate) use target_crate::*;` statement
    async fn add_internal_dependency(
        &self,
        target_crate: &H,
        dep_crate:    &H,
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl<P,H> AddInternalDependency<P,H> for Workspace<P,H> 
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Debug + Send + Sync,
{
    type Error = WorkspaceError;

    async fn add_internal_dependency(
        &self,
        target_crate: &H,
        dep_crate:    &H,
    ) -> Result<(), WorkspaceError> {
        use std::io::Write as _;
        use toml_edit::Document;
        
        info!("Beginning add_internal_dependency: from '{}' into '{}'",
              dep_crate.name(), target_crate.name());

        // 1) Compute the relative path from the target crate to the dep crate
        let target_absolute = target_crate
            .root_dir_path_buf()
            .canonicalize()
            .map_err(|e| {
                error!("Failed to canonicalize target_crate path: {:?}", e);
                WorkspaceError::IoError {
                    io_error: Arc::new(e),
                    context: format!("canonicalizing path for target_crate at {:?}", target_crate.root_dir_path_buf()),
                }
            })?;
        let dep_absolute = dep_crate
            .root_dir_path_buf()
            .canonicalize()
            .map_err(|e| {
                error!("Failed to canonicalize dep_crate path: {:?}", e);
                WorkspaceError::IoError {
                    io_error: Arc::new(e),
                    context: format!("canonicalizing path for dep_crate at {:?}", dep_crate.root_dir_path_buf()),
                }
            })?;
        let rel_path = pathdiff::diff_paths(&dep_absolute, &target_absolute)
            .unwrap_or_else(|| {
                warn!("Could not compute relative path; using absolute path fallback");
                dep_absolute.clone()
            });

        debug!("Relative path from target_crate to dep_crate is: {:?}", rel_path);

        // 2) Load the target crate's Cargo.toml
        let cargo_toml_path = target_absolute.join("Cargo.toml");
        trace!("Reading Cargo.toml from {:?}", cargo_toml_path);
        let existing_text = tokio::fs::read_to_string(&cargo_toml_path)
            .await
            .map_err(|io_err| {
                error!("Failed reading target's Cargo.toml: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: format!("reading target crate Cargo.toml at {:?}", cargo_toml_path),
                }
            })?;
        
        // 3) Parse with toml_edit and insert new dependency
        let mut doc = existing_text
            .parse::<Document>()
            .map_err(|toml_err| {
                error!("Failed to parse Cargo.toml as toml_edit::Document: {:?}", toml_err);
                CargoTomlError::TomlEditError {
                    cargo_toml_file: cargo_toml_path.clone(),
                    toml_parse_error: toml_err
                }
            })?;

        let dep_key = dep_crate.name().to_kebab_case();
        debug!("Adding dependency under [dependencies] with key = {:?}", dep_key);

        // Ensure we have a [dependencies] table
        if doc.get("dependencies").is_none() {
            doc["dependencies"] = toml_edit::Item::Table(toml_edit::Table::new());
        }

        // Insert the path-based dependency
        // e.g.,  dep_name = { path = "..." }
        doc["dependencies"][&*dep_key]["path"] = toml_edit::value(rel_path.to_string_lossy().to_string());

        // 4) Write the updated Cargo.toml back to disk
        let edited_toml = doc.to_string();
        trace!("Writing updated Cargo.toml with new dependency:\n{}", edited_toml);
        tokio::fs::write(&cargo_toml_path, edited_toml)
            .await
            .map_err(|io_err| {
                error!("Failed writing updated Cargo.toml: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: format!("writing updated Cargo.toml at {:?}", cargo_toml_path),
                }
            })?;

        // 5) Update or create src/imports.rs in the target crate
        let imports_file = target_absolute.join("src").join("imports.rs");
        debug!("Attempting to update imports file at {:?}", imports_file);

        // We'll read the current contents (if any), then append the needed pub(crate) use
        let mut existing_imports = String::new();
        match tokio::fs::read_to_string(&imports_file).await {
            Ok(contents) => {
                existing_imports = contents;
            },
            Err(e) => {
                warn!("Could not read existing imports file at {:?}: {:?}", imports_file, e);
                info!("Will create a new imports.rs file at {:?}", imports_file);
            }
        }

        let line_to_add = format!("pub(crate) use {}::*;\n", dep_key.to_snake_case());

        // If it's already present, skip. Otherwise, append.
        if !existing_imports.contains(&line_to_add) {
            debug!("Appending new line: {:?}", line_to_add.trim());
            existing_imports.push_str(&line_to_add);
            tokio::fs::create_dir_all(imports_file.parent().unwrap())
                .await
                .map_err(|io_err| {
                    error!("Failed creating directories for imports.rs: {:?}", io_err);
                    WorkspaceError::IoError {
                        io_error: Arc::new(io_err),
                        context: format!("create_dir_all for {:?}", imports_file.parent().unwrap()),
                    }
                })?;
            let mut file = tokio::fs::File::create(&imports_file).await.map_err(|io_err| {
                error!("Failed to create or open imports.rs: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: format!("creating imports.rs at {:?}", imports_file),
                }
            })?;
            file.write_all(existing_imports.as_bytes()).await.map_err(|io_err| {
                error!("Failed writing to imports.rs: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: format!("writing new line to {:?}", imports_file),
                }
            })?;
        } else {
            info!("Line already present in imports.rs, skipping append.");
        }

        info!("Successfully added internal dependency '{}' to '{}'",
              dep_crate.name(), target_crate.name());
        Ok(())
    }
}

#[cfg(test)]
mod test_add_internal_dependency {
    use super::*;

    /// A small helper that locks all `Arc<AsyncMutex<H>>` crate items in the workspace
    /// once and returns them in a `Vec<H>`. This lets us directly call `.name()`, 
    /// `.root_dir_path_buf()`, etc. on each crate handle rather than on the `Arc<Mutex<>>`.
    async fn ephemeral_handles<P, H>(ws: &Workspace<P, H>) -> Vec<H>
    where
        for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
        H: CrateHandleInterface<P> + Debug + Send + Sync + Clone,
    {
        let mut result = Vec::new();
        for arc_h in ws.crates() {
            let locked = arc_h.lock().await;
            result.push(locked.clone());
        }
        result
    }

    /// 1) Tests that adding a dependency to a crate successfully creates/updates
    ///    the [dependencies] in Cargo.toml and appends the `pub(crate) use dep_crate::*;` line
    ///    to `src/imports.rs`.
    #[tokio::test]
    async fn test_add_internal_dependency_happy_path() -> Result<(), WorkspaceError> {
        info!("Starting test_add_internal_dependency_happy_path");
        // 1) Create a mock workspace with two crates: A & B
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crateA").with_src_files(),
            CrateConfig::new("crateB").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        // 2) Build the workspace
        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should create a valid Workspace from path");

        // 3) Lock the crate handles once into ephemeral handles
        let ephemeral = ephemeral_handles(&ws).await;
        let crate_a = ephemeral
            .iter()
            .find(|h| h.name() == "crateA")
            .expect("Expected crateA in workspace");
        let crate_b = ephemeral
            .iter()
            .find(|h| h.name() == "crateB")
            .expect("Expected crateB in workspace");

        // 4) Perform the operation: add_internal_dependency
        ws.add_internal_dependency(crate_a, crate_b).await?;

        // 5) Now verify that crateA's Cargo.toml has a dependency on crateB
        let cargo_toml_a_path = crate_a.cargo_toml_path_buf().await?;
        debug!("Reading updated Cargo.toml at {:?}", cargo_toml_a_path);
        let updated_toml_a = fs::read_to_string(&cargo_toml_a_path).await
            .expect("Failed to read updated Cargo.toml for crateA");
        debug!("Updated Cargo.toml:\n{}", updated_toml_a);

        assert!(
            updated_toml_a.contains("[dependencies]"),
            "Should contain a [dependencies] section"
        );
        assert!(
            updated_toml_a.contains("crate-b = { path = "),
            "Should contain a path-based dependency entry for crateB"
        );

        // 6) Verify that crateA's src/imports.rs now has a pub(crate) use crateB
        let imports_rs_a = crate_a.root_dir_path_buf().join("src").join("imports.rs");
        debug!("Reading updated imports.rs at {:?}", imports_rs_a);
        let imports_contents = fs::read_to_string(&imports_rs_a).await
            .expect("Failed to read updated imports.rs for crateA");
        debug!("Updated imports.rs:\n{}", imports_contents);

        assert!(
            imports_contents.contains("pub(crate) use crate_b::*;"),
            "Should have appended `pub(crate) use crate_b::*;` line to imports.rs"
        );

        info!("test_add_internal_dependency_happy_path passed");
        Ok(())
    }

    /// 2) Tests that if the `src/imports.rs` file already has the desired
    ///    `pub(crate) use crateB::*;` line, we do not duplicate it.
    #[tokio::test]
    async fn test_add_internal_dependency_existing_import_line() -> Result<(), WorkspaceError> {
        info!("Starting test_add_internal_dependency_existing_import_line");
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("crateX").with_src_files(),
            CrateConfig::new("crateY").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should create a valid Workspace");

        // Lock ephemeral crate handles
        let ephemeral = ephemeral_handles(&ws).await;
        let crate_x = ephemeral
            .iter()
            .find(|h| h.name() == "crateX")
            .expect("Expected crateX in workspace");
        let crate_y = ephemeral
            .iter()
            .find(|h| h.name() == "crateY")
            .expect("Expected crateY in workspace");

        // We'll add the line to crateX's src/imports.rs manually
        let imports_rs = crate_x.root_dir_path_buf().join("src").join("imports.rs");
        fs::create_dir_all(imports_rs.parent().unwrap())
            .await
            .expect("Failed to create src dir");
        fs::write(&imports_rs, "pub(crate) use crateY::*;\n")
            .await
            .expect("Failed to write initial imports.rs with existing line");

        // Add the dependency
        ws.add_internal_dependency(crate_x, crate_y)
            .await?;

        // Verify we did NOT duplicate the line
        let updated_imports = fs::read_to_string(&imports_rs).await?;
        let count_matches = updated_imports.matches("pub(crate) use crateY::*;").count();
        assert_eq!(
            count_matches, 
            1, 
            "Should have exactly one line for crateY"
        );

        info!("test_add_internal_dependency_existing_import_line passed");
        Ok(())
    }

    /// 3) Tests that if the `[dependencies]` table is missing in Cargo.toml,
    ///    we create it before inserting the new dependency.
    #[tokio::test]
    async fn test_add_internal_dependency_creates_dependencies_table() -> Result<(), WorkspaceError> {
        info!("Starting test_add_internal_dependency_creates_dependencies_table");

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("alpha").with_src_files(),
            CrateConfig::new("beta").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        // We'll remove the entire `[dependencies]` table from alpha's Cargo.toml
        let alpha_cargo_path = workspace_path.join("alpha").join("Cargo.toml");
        let cargo_contents = fs::read_to_string(&alpha_cargo_path)
            .await
            .expect("Failed reading alpha Cargo.toml");
        let sanitized = cargo_contents.replace("[dependencies]", "");
        fs::write(&alpha_cargo_path, sanitized)
            .await
            .expect("Failed rewriting alpha Cargo.toml without [dependencies]");

        // Build the workspace
        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace after manual cargo edits");

        let ephemeral = ephemeral_handles(&ws).await;
        let alpha_crate = ephemeral
            .iter()
            .find(|h| h.name() == "alpha")
            .expect("Should find alpha crate");
        let beta_crate = ephemeral
            .iter()
            .find(|h| h.name() == "beta")
            .expect("Should find beta crate");

        // Now call add_internal_dependency
        ws.add_internal_dependency(alpha_crate, beta_crate).await?;

        // Check the updated Cargo.toml for alpha
        let updated = fs::read_to_string(&alpha_cargo_path).await?;
        debug!("Updated alpha Cargo.toml:\n{}", updated);

        assert!(
            updated.contains("[dependencies]"),
            "Should now have a [dependencies] table"
        );
        assert!(
            updated.contains("beta = { path = "),
            "Should have a path-based dependency on beta"
        );

        info!("test_add_internal_dependency_creates_dependencies_table passed");
        Ok(())
    }

    /// 4) Tests scenario where we have a crate recognized by the workspace at creation,
    ///    and then its Cargo.toml is removed. Reading it for add_internal_dependency
    ///    should fail with an IoError containing context.
    #[tokio::test]
    async fn test_add_internal_dependency_missing_cargo_toml() -> Result<(), CrateError> {
        info!("Starting test_add_internal_dependency_missing_cargo_toml");

        // Create a workspace with two crates: "good_crate" and "broken_crate"
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("good_crate").with_src_files(),
            CrateConfig::new("broken_crate").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        // First, build the workspace. This ensures both crates are recognized 
        // by the workspace since they have Cargo.toml files at creation time.
        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should create a valid Workspace with both crates recognized");

        // Lock ephemeral handles
        let ephemeral = ephemeral_handles(&ws).await;
        let good = ephemeral
            .iter()
            .find(|h| h.name() == "good_crate")
            .expect("Expected good_crate in workspace");
        let broken = ephemeral
            .iter()
            .find(|h| h.name() == "broken_crate")
            .expect("Expected broken_crate in workspace");

        // Now remove broken_crate's Cargo.toml AFTER the workspace is constructed
        let broken_cargo = broken.cargo_toml_path_buf().await?;
        fs::remove_file(&broken_cargo)
            .await
            .expect("Failed removing broken crate's Cargo.toml to simulate missing file");

        // Attempt to add dependency from broken->good => fails reading broken's Cargo.toml
        let result = ws.add_internal_dependency(broken, good).await;
        assert!(result.is_err(), "Should fail if Cargo.toml is missing for broken crate");

        match result.err().unwrap() {
            WorkspaceError::IoError { context, .. } => {
                assert!(
                    context.contains("reading target crate Cargo.toml"),
                    "Should have IoError with context about reading target crate Cargo.toml"
                );
            }
            other => {
                panic!("Expected IoError but got: {:?}", other);
            }
        }

        info!("test_add_internal_dependency_missing_cargo_toml passed");
        Ok(())
    }
}
