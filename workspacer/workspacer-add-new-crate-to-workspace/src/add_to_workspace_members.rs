// ---------------- [ File: workspacer-add-new-crate-to-workspace/src/add_to_workspace_members.rs ]
crate::ix!();

#[async_trait]
pub trait AddToWorkspaceMembers<P> 
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + Clone + 'static,
{
    async fn add_to_workspace_members(
        &self,
        new_crate_path: &P,
    ) -> Result<(), WorkspaceError>;
}

#[async_trait]
impl<P,H> AddToWorkspaceMembers<P> for Workspace<P,H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + Clone + 'static,
    H: CrateHandleInterface<P> + Send + Sync + 'static,
{
    async fn add_to_workspace_members(
        &self,
        new_crate_path: &P,
    ) -> Result<(), WorkspaceError> {
        info!("add_to_workspace_members - start for path='{}'", new_crate_path.as_ref().display());

        let top_cargo = self.as_ref().join("Cargo.toml");
        if !top_cargo.exists() {
            warn!("No top-level Cargo.toml found => skipping workspace membership update");
            return Ok(());
        }

        let contents = fs::read_to_string(&top_cargo).await.map_err(|e| {
            WorkspaceError::IoError {
                io_error: Arc::new(e),
                context: "reading top-level Cargo.toml".into()
            }
        })?;

        let mut doc = contents.parse::<TomlEditDocument>().map_err(|toml_err| {
            CargoTomlError::TomlEditError {
                cargo_toml_file: top_cargo.clone(),
                toml_parse_error: toml_err,
            }
        })?;

        // Check if we even have [workspace]
        let Some(workspace_item) = doc.get_mut("workspace") else {
            warn!("top-level Cargo.toml lacks [workspace]; skipping membership update");
            return Ok(());
        };

        // Ensure "workspace" is actually a table
        let Some(workspace_table) = workspace_item.as_table_mut() else {
            warn!("top-level Cargo.toml has 'workspace' but not a table; skipping membership update");
            return Ok(());
        };

        // Now ensure [workspace].members exists and is an array
        if workspace_table.get("members").is_none() {
            debug!("No 'members' found under [workspace], creating a fresh array");
            workspace_table.insert("members", TomlEditItem::Value(TomlEditValue::Array(TomlEditArray::new())));
        }

        let Some(members_item) = workspace_table.get_mut("members") else {
            // Should never happen if we just inserted it
            warn!("Failed to retrieve or create [workspace].members - skipping");
            return Ok(());
        };

        let Some(members_arr) = members_item.as_array_mut() else {
            // If it's not an array, we error
            return Err(WorkspaceError::InvalidCargoToml(
                CargoTomlError::CargoTomlWriteError(
                    CargoTomlWriteError::WriteWorkspaceMember {
                        io: Arc::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "[workspace].members is not an array"
                        ))
                    }
                )
            ));
        };

        // Compute a relative path from workspace root to new_crate_path
        let rel_path = new_crate_path.as_ref()
            .strip_prefix(self.as_ref())
            .unwrap_or(new_crate_path.as_ref())
            .to_string_lossy()
            .replace("\\", "/"); // for Windows

        // If not already in the array, push it
        let already_present = members_arr.iter().any(|itm| itm.as_str() == Some(&rel_path));
        if already_present {
            debug!("New crate path='{}' already in [workspace].members => skipping", rel_path);
        } else {
            debug!("Appending '{}' to [workspace].members array", rel_path);
            members_arr.push(&rel_path);
            let updated = doc.to_string();
            fs::write(&top_cargo, updated).await.map_err(|e| {
                WorkspaceError::IoError {
                    io_error: Arc::new(e),
                    context: format!("writing top-level Cargo.toml after adding member={}", rel_path)
                }
            })?;
        }

        info!("Successfully updated workspace members with '{}'", rel_path);
        Ok(())
    }
}

#[cfg(test)]
mod test_add_to_workspace_members {
    use super::*;

    ///
    /// Exhaustive tests for the `AddToWorkspaceMembers` extension trait. We verify:
    ///
    /// 1) **No top-level `Cargo.toml`** => logs a warning, returns `Ok(())` with no update
    /// 2) **No `[workspace]` table** => logs a warning, returns `Ok(())` with no update
    /// 3) `[workspace].members` is missing => we create it with an empty array, then append
    /// 4) If the new crate path is already present, we skip duplication
    /// 5) If it isn't present, we append to the array, then write the updated Cargo.toml
    /// 6) If writing fails, we get an `IoError` with context
    /// 7) If `[workspace].members` is not an array, we fail with `CargoTomlWriteError`
    ///
    /// We do real disk ops using `create_mock_workspace` to produce a top-level `Cargo.toml`
    /// with `[workspace]`. Then we read/modify it in tests to simulate or check behaviors. 
    ///

    // For convenience in these tests:
    type MyWorkspace = Workspace<PathBuf, CrateHandle>;

    // -------------------------------------------------------------------------
    // 1) Test: No top-level Cargo.toml => logs a warning, returns Ok
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_no_top_level_cargo_toml() {
        info!("Scenario 1: no top-level Cargo.toml => logs warning, returns Ok(())");

        // We'll create a mock workspace with 1 crate, then remove the root Cargo.toml entirely
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("one_crate").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace with one_crate");

        // Remove the top-level Cargo.toml
        let top_cargo = workspace_path.join("Cargo.toml");
        fs::remove_file(&top_cargo)
            .await
            .expect("Removed top-level Cargo.toml to simulate it's missing");

        // NB: The real `Workspace::new(...)` might fail if it insists on a valid `[workspace]`.
        // In that case, you won't have a functioning workspace object. If your code absolutely
        // refuses to parse a directory with no top-level Cargo.toml, you can skip this scenario
        // or wrap in a conditional test. We'll do a direct struct build here for demonstration:
        let ws: Workspace<PathBuf,CrateHandle> = WorkspaceBuilder::default()
            .path(&workspace_path)
            .crates(vec![])
            .build()
            .unwrap();

        // Now call `add_to_workspace_members`
        let new_crate_path = workspace_path.join("another_crate");
        let result = ws.add_to_workspace_members(&new_crate_path.into()).await;
        // We expect Ok(()) with a warning
        assert!(result.is_ok(),
            "Should return Ok if there's no top-level Cargo.toml => skipping membership update with a warn!");
    }

    // -------------------------------------------------------------------------
    // 2) Test: No [workspace] in top-level => logs warning, returns Ok
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_no_workspace_table_in_root() {
        info!("Scenario 2: top-level Cargo.toml has no [workspace], logs warning, returns Ok(())");

        // We'll create an empty workspace by default:
        let workspace_path = create_mock_workspace(vec![])
            .await
            .expect("Empty workspace creation");
        let root_cargo = workspace_path.join("Cargo.toml");

        // Overwrite top-level Cargo.toml so it has no [workspace]
        let minimal_toml = r#"
            [package]
            name = "top_level"
            version = "0.0.1"
        "#;
        fs::write(&root_cargo, minimal_toml).await
            .expect("Rewrite top-level cargo to remove [workspace] entirely");

        // We'll build a minimal workspace object (bypassing strict checks)
        let ws: Workspace<PathBuf,CrateHandle> = WorkspaceBuilder::default()
            .path(&workspace_path)
            .crates(vec![])
            .build()
            .unwrap();

        // Attempt to add a new crate path
        let new_crate_path = workspace_path.join("some_new_crate");
        let result = ws.add_to_workspace_members(&new_crate_path.into()).await;
        assert!(
            result.is_ok(),
            "Should log a warning, skip membership update, and return Ok if no [workspace] table"
        );
    }

    // -------------------------------------------------------------------------
    // 3) `[workspace].members` is missing => we create array, then append
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_missing_members_field_creates_array() {
        info!("Scenario 3: top-level cargo has [workspace] but no members => we create an array and add the new crate path.");

        // We'll create an empty workspace. By default, `create_mock_workspace` puts 
        // `[workspace]\nmembers=[ ... ]` in the top-level. Let's forcibly remove the members field entirely:
        let workspace_path = create_mock_workspace(vec![])
            .await
            .expect("Empty workspace ok");
        let root_cargo = workspace_path.join("Cargo.toml");

        // We'll parse with toml_edit, remove the "members" key entirely from [workspace]
        {
            let original = fs::read_to_string(&root_cargo).await.unwrap();
            let mut doc = original.parse::<TomlEditDocument>().unwrap();

            // Ensure doc["workspace"].exists, then remove the "members" key
            if let Some(ws_table) = doc.get_mut("workspace").and_then(|i| i.as_table_mut()) {
                ws_table.remove("members");
            }
            fs::write(&root_cargo, doc.to_string()).await.unwrap();
        }

        // Now we have a top-level cargo with [workspace], but no "members".
        let ws: Workspace<PathBuf,CrateHandle> = WorkspaceBuilder::default()
            .path(&workspace_path)
            .crates(vec![])
            .build()
            .unwrap();


        // 2) Attempt to add a new crate path
        let new_crate_path = workspace_path.join("some_crate");
        ws.add_to_workspace_members(&new_crate_path.into())
            .await
            .expect("Should succeed creating members array and appending the new crate path");

        // 3) Check updated Cargo.toml
        let updated_txt = fs::read_to_string(&root_cargo).await.unwrap();
        debug!("Updated top-level cargo:\n{}", updated_txt);
        // We expect to see:
        // [workspace]
        // members = ["some_crate"]
        assert!(updated_txt.contains("[workspace]"), "Should still have a [workspace] table");
        assert!(updated_txt.contains("\"some_crate\""),
            "Should have appended \"some_crate\" in the members array");
    }

    // -------------------------------------------------------------------------
    // 4) Already present => skip duplication
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_already_present_skip_dup() {
        info!("Scenario 4: The crate path is already in workspace.members => skip duplication");

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("an_existing_crate").with_src_files(),
        ])
        .await
        .expect("Mock workspace with 'an_existing_crate' => so top-level cargo includes it in members");

        let root_cargo = workspace_path.join("Cargo.toml");
        // We'll parse & ensure the membership array includes "some_crate" so we have a duplicate scenario
        {
            let original = fs::read_to_string(&root_cargo).await.unwrap();
            let mut doc = original.parse::<TomlEditDocument>().unwrap();

            // doc["workspace"]["members"] should be an array. We'll push "some_crate".
            let arr = doc["workspace"]["members"]
                .as_array_mut()
                .expect("create_mock_workspace always sets up an array for members");
            arr.push("some_crate");
            fs::write(&root_cargo, doc.to_string()).await.unwrap();
        }

        // Build minimal workspace object
        let ws: Workspace<PathBuf,CrateHandle> = WorkspaceBuilder::default()
            .path(&workspace_path)
            .crates(vec![])
            .build()
            .unwrap();

        // Now call add_to_workspace_members with "some_crate"
        let crate_path = workspace_path.join("some_crate");
        ws.add_to_workspace_members(&crate_path.into())
            .await
            .expect("Should skip duplication, returning Ok");

        // Re-read top-level cargo => ensure "some_crate" is present exactly once
        let updated_txt = fs::read_to_string(&root_cargo).await.unwrap();
        let count = updated_txt.matches("some_crate").count();
        assert_eq!(
            count, 1,
            "Should only appear once in the members array, skipping duplication"
        );
    }

    // -------------------------------------------------------------------------
    // 5) Not present => we append, rewriting
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_not_present_we_append() {
        info!("Scenario 5: new crate path not present => we append it, rewriting cargo");

        // We'll create a minimal workspace with 1 crate: "alpha"
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("alpha").with_src_files(),
        ])
        .await
        .expect("Workspace with 'alpha' crate");

        let root_cargo = workspace_path.join("Cargo.toml");
        let old_txt = fs::read_to_string(&root_cargo).await.unwrap();
        assert!(old_txt.contains("alpha"), "Should have 'alpha' in members array initially");

        // We'll define a second crate path "beta"
        let ws: Workspace<PathBuf,CrateHandle> = WorkspaceBuilder::default()
            .path(&workspace_path)
            .crates(vec![])
            .build()
            .unwrap();

        let beta_path = workspace_path.join("beta");

        ws.add_to_workspace_members(&beta_path.into())
            .await
            .expect("Should succeed appending 'beta' to members array");

        let updated_txt = fs::read_to_string(&root_cargo).await.unwrap();
        debug!("Updated top-level cargo:\n{}", updated_txt);
        assert!(
            updated_txt.contains("beta"),
            "Should contain 'beta' in the members array"
        );
    }

    // -------------------------------------------------------------------------
    // 6) If writing fails, we get IoError with context
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_writing_fails_ioerror() {
        info!("Scenario 6: rewriting the top-level cargo fails => we get IoError with context.");

        #[cfg(target_os = "linux")]
        {
            // We'll create an empty workspace
            let workspace_path = create_mock_workspace(vec![])
                .await
                .expect("Empty workspace ok");
            let root_cargo = workspace_path.join("Cargo.toml");

            // Set the file to read-only
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&root_cargo).await.unwrap().permissions();
            perms.set_mode(0o444); // read-only
            fs::set_permissions(&root_cargo, perms).await.unwrap();

            // Build a minimal workspace object
            let ws: Workspace<PathBuf,CrateHandle> = WorkspaceBuilder::default()
                .path(&workspace_path)
                .crates(vec![])
                .build()
                .unwrap();

            // Now attempt to add a new crate => rewriting top-level cargo should fail
            let new_crate_path = workspace_path.join("some_new_crate");
            let result = ws.add_to_workspace_members(&new_crate_path.into()).await;
            match result {
                Err(WorkspaceError::IoError { context, .. }) => {
                    assert!(
                        context.contains("writing top-level Cargo.toml after adding member=some_new_crate"),
                        "Should mention rewriting top-level cargo in the context"
                    );
                    info!("Got expected IoError for read-only cargo toml");
                },
                Ok(_) => warn!("Unexpected success rewriting read-only cargo => environment might not enforce perms"),
                other => panic!("Unexpected error: {:?}", other),
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            info!("Skipping test_writing_fails_ioerror on non-Linux if permissions are not enforced similarly.");
        }
    }

    // -------------------------------------------------------------------------
    // 7) If `[workspace].members` is not an array, fail with `CargoTomlWriteError`
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_members_field_not_array() {
        info!("Scenario 7: `[workspace].members` is not an array => we fail with CargoTomlWriteError");

        let workspace_path = create_mock_workspace(vec![])
            .await
            .expect("Empty workspace ok");
        let root_cargo = workspace_path.join("Cargo.toml");

        // Overwrite so that `[workspace].members` is a string => not an array
        {
            let original = fs::read_to_string(&root_cargo).await.unwrap();
            let mut doc = original.parse::<TomlEditDocument>().unwrap();

            // doc["workspace"] presumably is a table. We'll forcibly set `members` to a string
            doc["workspace"]["members"] = TomlEditItem::Value(TomlEditValue::from("not-an-array"));
            fs::write(&root_cargo, doc.to_string()).await.unwrap();
        }

        // Build a minimal workspace
        let ws: Workspace<PathBuf,CrateHandle> = WorkspaceBuilder::default()
            .path(&workspace_path)
            .crates(vec![])
            .build()
            .unwrap();

        // Attempt to add "unknown"
        let crate_path = workspace_path.join("unknown");
        let result = ws.add_to_workspace_members(&crate_path.into()).await;

        // We expect an error: CargoTomlWriteError => "workspace.members not an array"
        match result {
            Err(WorkspaceError::InvalidCargoToml(
                CargoTomlError::CargoTomlWriteError(CargoTomlWriteError::WriteWorkspaceMember { .. })
            )) => {
                info!("Got the expected CargoTomlWriteError for 'workspace.members not an array'");
            },
            other => panic!("Expected a CargoTomlWriteError about members not an array, got: {:?}", other),
        }
    }
}
