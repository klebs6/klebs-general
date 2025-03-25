// ---------------- [ File: workspacer-register-internal-crate-in-prefix-group/src/register_internal_crate_in_prefix_group.rs ]
crate::ix!();

/// This trait says: "Given a prefix group’s facade crate, register a new internal crate in it."
/// Typically means:
/// 1) Add a `[dependencies] new_crate = { path = ... }` to the facade crate’s Cargo.toml.
/// 2) Possibly add `pub use new_crate::*;` or a mod statement in facade crate’s code.
///
#[async_trait]
pub trait RegisterInPrefixGroup<P,H> 
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
{
    type Error;
    async fn register_in_prefix_crate(
        &self,
        prefix_crate: &H,
        new_crate:    &H,
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl<P,H> RegisterInPrefixGroup<P,H> for Workspace<P,H>
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
{
    type Error = WorkspaceError;

    ///
    /// Registers `new_crate` into the specified `prefix_crate` by:
    /// 1) **Adding** a path-based dependency in `prefix_crate`'s Cargo.toml under `[dependencies] new_crate = { path="..." }`
    /// 2) **Inserting** a `pub use new_crate_identifier::*;` line in `prefix_crate`'s `src/lib.rs`
    ///    (Only if such a line doesn't already exist).
    ///
    /// * "new_crate_identifier" is typically the crate's name with `-` replaced by `_`, e.g. "batch-mode-batch-schema"
    ///   => "batch_mode_batch_schema".
    ///
    /// ### Steps:
    /// - **Compute** the relative path from `prefix_crate` to `new_crate` for Cargo.toml
    /// - **Load** `prefix_crate`'s Cargo.toml via `toml_edit` and insert dependency
    /// - **Write** updated Cargo.toml
    /// - **Append** (if not already present) a `pub use <new_crate_identifier>::*;` in `prefix_crate`'s `src/lib.rs`
    ///
    async fn register_in_prefix_crate(
        &self,
        prefix_crate: &H,
        new_crate:    &H,
    ) -> Result<(), Self::Error> {
        use std::io::Write as _;
        use toml_edit::Document;

        info!(
            "Registering crate '{}' into prefix crate '{}'",
            new_crate.name(), prefix_crate.name()
        );

        // 1) Compute relative path from prefix_crate to new_crate
        let prefix_abs = prefix_crate
            .root_dir_path_buf()
            .canonicalize()
            .map_err(|e| {
                error!("Failed to canonicalize prefix_crate path: {:?}", e);
                WorkspaceError::IoError {
                    io_error: Arc::new(e),
                    context: format!("canonicalizing path for prefix_crate at {:?}", prefix_crate.root_dir_path_buf()),
                }
            })?;
        let new_abs = new_crate
            .root_dir_path_buf()
            .canonicalize()
            .map_err(|e| {
                error!("Failed to canonicalize new_crate path: {:?}", e);
                WorkspaceError::IoError {
                    io_error: Arc::new(e),
                    context: format!("canonicalizing path for new_crate at {:?}", new_crate.root_dir_path_buf()),
                }
            })?;

        let rel_path = pathdiff::diff_paths(&new_abs, &prefix_abs)
            .unwrap_or_else(|| {
                warn!("Could not compute relative path; using absolute path fallback");
                new_abs.clone()
            });

        debug!("Relative path from prefix_crate => new_crate is {:?}", rel_path);

        // 2) Load prefix_crate's Cargo.toml and insert a dependency
        let cargo_toml_path = prefix_abs.join("Cargo.toml");
        debug!("Reading Cargo.toml from {:?}", cargo_toml_path);

        let existing_text = tokio::fs::read_to_string(&cargo_toml_path)
            .await
            .map_err(|io_err| {
                error!("Failed reading prefix_crate Cargo.toml: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: format!("reading prefix_crate Cargo.toml at {:?}", cargo_toml_path),
                }
            })?;

        let mut doc = existing_text
            .parse::<Document>()
            .map_err(|toml_err| {
                error!("Failed to parse Cargo.toml as toml_edit::Document: {:?}", toml_err);
                CargoTomlError::TomlEditError {
                    cargo_toml_file: cargo_toml_path.clone(),
                    toml_parse_error: toml_err
                }
            })?;

        let dep_key = new_crate.name();
        debug!("Adding dependency under [dependencies] with key = {:?}", dep_key);

        // Ensure we have a [dependencies] table
        if doc.get("dependencies").is_none() {
            doc["dependencies"] = toml_edit::Item::Table(toml_edit::Table::new());
        }

        // Insert path-based dependency
        doc["dependencies"][&*dep_key]["path"] = toml_edit::value(rel_path.to_string_lossy().to_string());

        // 3) Write the updated Cargo.toml back
        let edited_toml = doc.to_string();
        trace!("Writing updated Cargo.toml:\n{}", edited_toml);
        tokio::fs::write(&cargo_toml_path, edited_toml)
            .await
            .map_err(|io_err| {
                error!("Failed writing updated prefix_crate Cargo.toml: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: format!("writing updated Cargo.toml at {:?}", cargo_toml_path),
                }
            })?;

        // 4) Add a public re-export line in `prefix_crate`'s src/lib.rs
        // We'll transform dashes to underscores for Rust import path
        let new_crate_ident = dep_key.replace('-', "_"); 
        let reexport_line = format!("pub use {}::*;\n", new_crate_ident);

        let lib_rs_path = prefix_abs.join("src").join("lib.rs");
        debug!("Updating lib.rs at {:?}", lib_rs_path);

        // We'll read existing lib.rs (if any), then append the line if missing
        let mut existing_lib = String::new();
        match tokio::fs::read_to_string(&lib_rs_path).await {
            Ok(contents) => {
                existing_lib = contents;
            },
            Err(e) => {
                warn!("Could not read existing lib.rs at {:?}: {:?}", lib_rs_path, e);
                info!("Will create a new lib.rs file at {:?}", lib_rs_path);
            }
        }

        if !existing_lib.contains(&reexport_line) {
            debug!("Appending new line: {:?}", reexport_line.trim());
            existing_lib.push_str(&reexport_line);
            tokio::fs::create_dir_all(lib_rs_path.parent().unwrap())
                .await
                .map_err(|io_err| {
                    error!("Failed creating directories for lib.rs: {:?}", io_err);
                    WorkspaceError::IoError {
                        io_error: Arc::new(io_err),
                        context: format!("create_dir_all for {:?}", lib_rs_path.parent().unwrap()),
                    }
                })?;
            let mut file = tokio::fs::File::create(&lib_rs_path).await.map_err(|io_err| {
                error!("Failed to create or open lib.rs: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: format!("creating lib.rs at {:?}", lib_rs_path),
                }
            })?;
            use tokio::io::AsyncWriteExt;
            file.write_all(existing_lib.as_bytes()).await.map_err(|io_err| {
                error!("Failed writing to lib.rs: {:?}", io_err);
                WorkspaceError::IoError {
                    io_error: Arc::new(io_err),
                    context: format!("writing new line to {:?}", lib_rs_path),
                }
            })?;
        } else {
            info!("Re-export line already present in lib.rs, skipping append.");
        }

        info!("Successfully registered '{}' into prefix crate '{}'",
            new_crate.name(), prefix_crate.name()
        );
        Ok(())
    }
}

#[cfg(test)]
mod test_register_in_prefix_group {
    use super::*;

    /// A small helper that locks all crate Arc<AsyncMutex<H>> items and returns
    /// them in a new Vec. We can then do direct crate.name(), crate.root_dir_path_buf(), etc.
    async fn ephemeral_handles<P, H>(ws: &Workspace<P, H>) -> Vec<H>
    where
        for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
        H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
    {
        let mut result = Vec::new();
        for arc_h in ws.crates() {
            let locked = arc_h.lock().await;
            result.push(locked.clone());
        }
        result
    }

    ///
    /// Exhaustive tests for `register_in_prefix_crate(...)`, verifying that:
    ///
    /// 1) We correctly add a path-based dependency to the prefix crate’s Cargo.toml
    ///    under `[dependencies] <new_crate_name> = { path = "..." }`.
    /// 2) We convert any `-` in `<new_crate_name>` to underscores for the `pub use...;` line in `src/lib.rs`.
    /// 3) If `lib.rs` doesn’t exist, we create it; if `[dependencies]` doesn’t exist, we create it.
    /// 4) If the re-export line is already present, we skip adding a duplicate.
    /// 5) If an error occurs (e.g., missing prefix crate Cargo.toml), we return an appropriate `WorkspaceError`.
    ///
    /// We arrange multiple scenarios, including partial setups, missing files, etc.
    ///

    // -------------------------------------------------------------------------
    // 1) Basic scenario: prefix crate + new crate => success
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn test_basic_register_success() -> Result<(), WorkspaceError> {
        // We'll create a workspace with 2 crates:
        // prefix_crate: "batch-mode"
        // new_crate: "batch-mode-batch-schema"
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-batch-schema").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        // Lock each crate handle so we can do .name() directly.
        let ephemeral = ephemeral_handles(&ws).await;

        // Grab references by name
        let prefix_crate = ephemeral
            .iter()
            .find(|h| h.name() == "batch-mode")
            .expect("Expected batch-mode in workspace");
        let new_crate = ephemeral
            .iter()
            .find(|h| h.name() == "batch-mode-batch-schema")
            .expect("Expected batch-mode-batch-schema in workspace");

        // 1) Call register_in_prefix_crate
        ws.register_in_prefix_crate(prefix_crate, new_crate).await?;

        // 2) Verify the prefix crate's Cargo.toml => has [dependencies][batch-mode-batch-schema]
        let cargo_toml_path = prefix_crate.cargo_toml_path_buf().await?;
        let cargo_content = fs::read_to_string(&cargo_toml_path)
            .await
            .expect("Failed to read prefix crate Cargo.toml");
        assert!(
            cargo_content.contains("[dependencies]"),
            "Should contain a [dependencies] section"
        );
        assert!(
            cargo_content.contains("batch-mode-batch-schema = { path = "),
            "Should contain a path-based dependency entry for batch-mode-batch-schema"
        );

        // 3) Verify that `src/lib.rs` has `pub use batch_mode_batch_schema::*;`
        let lib_rs_path = prefix_crate.root_dir_path_buf().join("src").join("lib.rs");
        let lib_rs_content = fs::read_to_string(&lib_rs_path).await
            .expect("Failed to read prefix crate lib.rs");
        assert!(
            lib_rs_content.contains("pub use batch_mode_batch_schema::*;"),
            "Expected `pub use batch_mode_batch_schema::*;` line in lib.rs"
        );

        Ok(())
    }

    // -------------------------------------------------------------------------
    // 2) If lib.rs doesn't exist, we create it
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn test_missing_lib_rs_is_created() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("some-facade").with_src_files(),
            CrateConfig::new("some-facade-extra").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        // We'll delete the prefix crate's `src/lib.rs` to simulate no file
        let facade_dir = workspace_path.join("some-facade");
        let _ = fs::remove_file(facade_dir.join("src").join("lib.rs")).await;

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let ephemeral = ephemeral_handles(&ws).await;
        let prefix_crate = ephemeral
            .iter()
            .find(|h| h.name() == "some-facade")
            .expect("Expected some-facade");
        let new_crate = ephemeral
            .iter()
            .find(|h| h.name() == "some-facade-extra")
            .expect("Expected some-facade-extra");

        ws.register_in_prefix_crate(prefix_crate, new_crate).await?;

        // Check that lib.rs now exists
        let lib_rs_path = facade_dir.join("src").join("lib.rs");
        let lib_rs_content = fs::read_to_string(&lib_rs_path).await
            .expect("lib.rs should be created");
        assert!(
            lib_rs_content.contains("pub use some_facade_extra::*;"),
            "Should contain the re-export line in newly created lib.rs"
        );

        Ok(())
    }

    // -------------------------------------------------------------------------
    // 3) If the re-export line is already present, do not duplicate
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn test_already_has_reexport_line_no_duplicate() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("abc").with_src_files(),
            CrateConfig::new("abc-sub").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        let abc_dir = workspace_path.join("abc");
        fs::create_dir_all(abc_dir.join("src")).await.unwrap();

        // Write an existing line to lib.rs
        let existing_line = "pub use abc_sub::*;\n";
        let lib_rs_path = abc_dir.join("src").join("lib.rs");
        fs::write(&lib_rs_path, existing_line)
            .await
            .expect("Failed to write existing lib.rs content");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let ephemeral = ephemeral_handles(&ws).await;
        let prefix_crate = ephemeral
            .iter()
            .find(|h| h.name() == "abc")
            .expect("Expected abc crate");
        let new_crate = ephemeral
            .iter()
            .find(|h| h.name() == "abc-sub")
            .expect("Expected abc-sub crate");

        ws.register_in_prefix_crate(prefix_crate, new_crate).await?;

        // read lib.rs again
        let updated_lib = fs::read_to_string(&lib_rs_path).await.unwrap();
        let count_matches = updated_lib.matches("pub use abc_sub::*;").count();
        assert_eq!(
            count_matches, 1,
            "Should have exactly one line for `pub use abc_sub::*;`"
        );

        Ok(())
    }

    // -------------------------------------------------------------------------
    // 4) If prefix crate Cargo.toml has no [dependencies], we create it
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn test_no_dependencies_section() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("xyz").with_src_files(),
            CrateConfig::new("xyz-tool").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        // We'll remove any existing [dependencies] from xyz's Cargo.toml
        let xyz_cargo_path = workspace_path.join("xyz").join("Cargo.toml");
        let original = fs::read_to_string(&xyz_cargo_path).await.unwrap();
        let sanitized = original.replace("[dependencies]", "");
        fs::write(&xyz_cargo_path, sanitized).await.unwrap();

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let ephemeral = ephemeral_handles(&ws).await;
        let prefix_crate = ephemeral
            .iter()
            .find(|h| h.name() == "xyz")
            .expect("Expected xyz crate");
        let new_crate = ephemeral
            .iter()
            .find(|h| h.name() == "xyz-tool")
            .expect("Expected xyz-tool crate");

        ws.register_in_prefix_crate(prefix_crate, new_crate).await?;

        let updated_cargo = fs::read_to_string(&xyz_cargo_path).await.unwrap();
        assert!(
            updated_cargo.contains("[dependencies]"),
            "Should now have a [dependencies] table in xyz's Cargo.toml"
        );
        assert!(
            updated_cargo.contains("xyz-tool = { path = "),
            "Should have path-based dependency on xyz-tool"
        );
        Ok(())
    }

    // -------------------------------------------------------------------------
    // 5) Missing prefix crate Cargo.toml => IoError
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn test_missing_prefix_crate_cargo_toml() -> Result<(), ()> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("prefix_miss").with_src_files(),
            CrateConfig::new("prefix_miss_lib").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace with 2 crates");

        // Remove the prefix crate's Cargo.toml
        let prefix_cargo_path = workspace_path.join("prefix_miss").join("Cargo.toml");
        fs::remove_file(&prefix_cargo_path)
            .await
            .expect("Removed prefix_miss Cargo.toml to trigger IoError later");

        // Lock ephemeral
        let ephemeral = ephemeral_handles(&ws).await;
        let prefix_crate = ephemeral
            .iter()
            .find(|h| h.name() == "prefix_miss")
            .expect("prefix_miss recognized by workspace");
        let new_crate = ephemeral
            .iter()
            .find(|h| h.name() == "prefix_miss_lib")
            .expect("prefix_miss_lib recognized by workspace");

        // Attempt to register => fails
        let result = ws.register_in_prefix_crate(prefix_crate, new_crate).await;
        match result {
            Err(WorkspaceError::IoError { context, .. }) => {
                assert!(
                    context.contains("reading prefix_crate Cargo.toml"),
                    "Should mention reading prefix_crate Cargo.toml in the context"
                );
            }
            other => panic!("Expected IoError, got {other:?}"),
        }

        Ok(())
    }

    // -------------------------------------------------------------------------
    // 6) Check dash => underscore transformation in `lib.rs` re-export
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn test_dashes_to_underscores() -> Result<(), WorkspaceError> {
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("acme").with_src_files(),
            CrateConfig::new("acme-awesome-feature").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let ephemeral = ephemeral_handles(&ws).await;
        let prefix_crate = ephemeral
            .iter()
            .find(|h| h.name() == "acme")
            .expect("Expected acme in workspace");
        let new_crate = ephemeral
            .iter()
            .find(|h| h.name() == "acme-awesome-feature")
            .expect("Expected acme-awesome-feature in workspace");

        ws.register_in_prefix_crate(prefix_crate, new_crate).await?;

        // check lib.rs for "pub use acme_awesome_feature::*;"
        let lib_rs_path = prefix_crate.root_dir_path_buf().join("src").join("lib.rs");
        let lib_contents = fs::read_to_string(&lib_rs_path).await
            .expect("Failed to read prefix crate lib.rs");
        assert!(
            lib_contents.contains("pub use acme_awesome_feature::*;"),
            "Should contain underscores in re-export line"
        );

        Ok(())
    }
}
