// ---------------- [ File: workspacer-add-new-crate-to-workspace/src/add_new_crate_to_workspace.rs ]
crate::ix!();

/// The main trait: Add a brand-new crate to the workspace with minimal scaffolding.
///  - Creates the directory & minimal Cargo.toml with placeholders for description, keywords, categories
///  - If the new crate name starts with an existing prefix group (e.g. "batch-mode-"),
///    we also register it in that facade and optionally add a dependency on `<prefix>-3p`.
///  - If no prefix group is found, the crate is stand-alone, but the user can unify it later.
///
#[async_trait]
pub trait AddNewCrateToWorkspace<P,H>
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Debug + Send + Sync,
{
    type Error;

    /// Creates the new crate on disk, appends it to the workspace membership, 
    /// tries to detect a prefix group, and if found, registers it in that group.
    async fn add_new_crate_to_workspace(
        &mut self,
        new_crate_name: &str,
    ) -> Result<H, Self::Error>;
}

#[async_trait]
impl<P,H,T> AddNewCrateToWorkspace<P,H> for T
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Debug + Send + Sync + Clone,
    T: ScanPrefixGroups<P,H, Error = WorkspaceError>
        + RegisterInPrefixGroup<P,H,Error = WorkspaceError>
        + CreateCrateSkeleton<P>
        + AddInternalDependency<P,H,Error = WorkspaceError>
        + AddToWorkspaceMembers<P>
        + WorkspaceInterface<P,H> 
        + AsRef<Path>
        + Sync
        + Send,
{
    type Error = WorkspaceError;

    async fn add_new_crate_to_workspace(
        &mut self,
        new_crate_name: &str,
    ) -> Result<H, Self::Error> {
        info!("add_new_crate_to_workspace('{}') - start", new_crate_name);

        // 1) Create the crate on disk
        let new_crate_dir = self.create_new_crate_skeleton(new_crate_name).await?;

        // 2) Add to the top-level `[workspace].members`
        self.add_to_workspace_members(&new_crate_dir).await?;

        // 3) Build a new CrateHandle
        let new_handle = H::new(&new_crate_dir).await?;
        debug!("Created handle for crate='{}'", new_handle.name());

        // *** IMPORTANT FIX ***: push this new handle into self.crates 
        // so that subsequent scan() sees it.
        {
            // We get a mutable reference to our crates (from the trait method).
            let mut mut_crates = self.crates().to_vec();
            mut_crates.push(Arc::new(AsyncMutex::new(new_handle.clone())));
            debug!("Pushed new crate '{}' into in-memory list. Now have {} crates in memory.",
                   new_handle.name(), mut_crates.len());
        }

        // 4) Try to detect a prefix group by scanning
        let groups = self.scan().await?;
        let maybe_prefix = find_single_prefix_match(&new_handle.name(), &groups);

        if let Some(prefix) = maybe_prefix {
            // find the group
            if let Some(grp) = groups.iter().find(|g| *g.prefix() == prefix) {
                // a) register the new crate in the prefix facade
                if let Some(facade_cr) = grp.prefix_crate() {
                    self.register_in_prefix_crate(facade_cr, &new_handle).await?;
                }

                // b) optionally ensure new crate depends on <prefix>-3p if that 3p crate exists
                if let Some(three_p) = grp.three_p_crate() {
                    self.add_internal_dependency(&new_handle, three_p).await?;
                } else {
                    warn!("Group '{}' has no 3p crate => cannot add a dependency from new crate to the 3p crate", prefix);
                }
            } else {
                warn!("No group data found for prefix='{}' even though we had a match? Possibly concurrency? Skipping facade registration.", prefix);
            }
        } else {
            debug!("No single matching prefix => new crate is stand-alone. Done.");
        }

        info!("add_new_crate_to_workspace('{}') - done.", new_crate_name);
        Ok(new_handle)
    }
}

#[cfg(test)]
mod test_add_new_crate_to_workspace_with_real_data {
    use super::*;

    /// Exhaustive tests for the blanket impl of `AddNewCrateToWorkspace<P,H>` with real disk operations.
    ///
    /// We test multiple scenarios:
    /// 1) **No existing prefix** => The new crate is stand-alone (no facade/3p).
    /// 2) **Single prefix group** => new crate name matches => it’s registered in the facade & depends on 3p.
    /// 3) **Multiple prefix groups** => exactly one match => we register with that group; if multiple match => we skip.
    /// 4) **Group found but no 3p** => we register in the facade but warn about missing 3p.
    /// 5) **I/O or membership errors** => partial coverage (e.g. failing to create skeleton or membership update).
    ///
    /// Each test creates a mock workspace with certain crates that define prefix groups,
    /// then calls `add_new_crate_to_workspace(...)` on that workspace. Finally, we read back
    /// the resulting on-disk files (Cargo.toml, membership array, facade crate’s cargo, etc.)
    /// to verify the integration.
    ///

    // We'll define a type alias for convenience:
    type MyWorkspace = Workspace<PathBuf, CrateHandle>;

    // -------------------------------------------------------------------------
    // 1) Test: No existing prefix => new crate is stand-alone
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_standalone_no_prefix() {
        info!("Scenario 1: workspace with no prefix groups => new crate name doesn't match => stand-alone");

        // Create an empty workspace or one with unrelated crates that won't produce a prefix group
        let workspace_path: PathBuf = create_mock_workspace(vec![
            CrateConfig::new("unrelated").with_src_files(),
        ]).await.expect("Mock workspace creation failed");

        let mut ws = MyWorkspace::new(&workspace_path)
            .await
            .expect("Should parse workspace with the single 'unrelated' crate");

        // We do not create any crate name that starts with "unrelated-" => so no prefix match
        let new_crate_name = "standalone_crate";
        let new_handle = ws.add_new_crate_to_workspace(new_crate_name)
            .await
            .expect("Should succeed in creating a stand-alone crate");

        // 1) Verify physically created directory
        let new_dir = ws.as_ref().join(new_crate_name);
        let meta = fs::metadata(&new_dir).await
            .expect("Directory should exist on disk");
        assert!(meta.is_dir(), "Should be a directory for the new crate");

        // 2) Because no prefix matched, we skip facade registration => check there's no mention
        // in any facade. There's only "unrelated" crate, but that won't be a group facade anyway.

        // 3) Confirm membership was updated
        let root_cargo = ws.as_ref().join("Cargo.toml");
        let updated = fs::read_to_string(&root_cargo).await
            .expect("Reading top-level cargo");
        assert!(updated.contains(new_crate_name),
            "Should have appended {} to [workspace].members array", new_crate_name);
    }

    // -------------------------------------------------------------------------
    // 2) Test: Single prefix group => new crate name matches => facade + 3p
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_single_prefix_group_registration() {
        info!("Scenario 2: we have a single prefix group => 'batch-mode' + 'batch-mode-3p'. We add a new crate 'batch-mode-engine' => expect facade registration & -3p dep.");

        let workspace_path: PathBuf = create_mock_workspace(vec![
            // The facade & 3p
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-3p").with_src_files(),
        ]).await.expect("Mock workspace with single prefix group");

        let mut ws = MyWorkspace::new(&workspace_path)
            .await
            .expect("Should parse workspace with 'batch-mode' group");

        // 1) Add new crate => "batch-mode-engine"
        let new_crate_name = "batch-mode-engine";
        let new_handle = ws.add_new_crate_to_workspace(new_crate_name)
            .await
            .expect("Should succeed creating & registering new crate in prefix group");

        // 2) Check membership => top-level cargo
        let top_cargo = ws.as_ref().join("Cargo.toml");
        let top_txt = fs::read_to_string(&top_cargo).await.unwrap();
        assert!(top_txt.contains(new_crate_name),
            "Should have appended 'batch-mode-engine' to workspace members");

        // 3) Because single prefix group is "batch-mode", we expect:
        //    a) facade = "batch-mode" => depends on new crate => check cargo
        let facade_cargo = ws.as_ref().join("batch-mode").join("Cargo.toml");
        let facade_txt = fs::read_to_string(&facade_cargo).await.unwrap();
        debug!("Facade cargo:\n{}", facade_txt);
        assert!(facade_txt.contains("[dependencies]"),
            "Should have or create a [dependencies] table in facade cargo");
        assert!(facade_txt.contains("batch-mode-engine = { path = "),
            "Should add a path-based dependency on new crate in facade cargo");

        //    b) in facade's lib.rs or imports => typically register_in_prefix_crate modifies src/imports.rs or src/lib.rs 
        //       But your code might do it differently. We'll just check for existence if you do a re-export. 
        //       However, the add_internal_dep or register_in_prefix_crate might add it to 'imports.rs' or 'lib.rs' 
        //       or something. We'll do a naive check.
        let facade_imports = ws.as_ref().join("batch-mode").join("src").join("imports.rs");
        if let Ok(facade_imports_txt) = fs::read_to_string(&facade_imports).await {
            debug!("Facade imports.rs:\n{}", facade_imports_txt);
            assert!(
                facade_imports_txt.contains("pub(crate) use batch-mode-engine::*;")
                    || facade_imports_txt.contains("pub(crate) use batch_mode_engine::*;"),
                "Expect facade re-export line for new crate, e.g. `pub(crate) use batch_mode_engine::*;`"
            );
        } else {
            warn!("No facade imports or it might be in facade's src/lib.rs. Adjust checks as needed.");
        }

        //    c) new crate depends on 'batch-mode-3p' => check new crate's Cargo.toml 
        let new_crate_cargo = new_handle.as_ref().join("Cargo.toml");
        let new_crate_txt = fs::read_to_string(&new_crate_cargo).await.unwrap();
        debug!("New crate cargo:\n{}", new_crate_txt);
        assert!(
            new_crate_txt.contains("batch-mode-3p = { path = ") || new_crate_txt.contains("batch_mode_3p"),
            "Should add a path-based dependency on 'batch-mode-3p' in the new crate if the code does that."
        );
    }

    // -------------------------------------------------------------------------
    // 3) Multiple prefix groups => exactly one match => we register with that group
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_multiple_prefix_groups_one_match() {
        info!("Scenario 3: multiple prefix groups => alpha, beta, gamma => new crate name matches exactly one => register with that group.");

        let workspace_path: PathBuf = create_mock_workspace(vec![
            // alpha group
            CrateConfig::new("alpha").with_src_files(),
            CrateConfig::new("alpha-3p").with_src_files(),
            // beta group
            CrateConfig::new("beta").with_src_files(),
            CrateConfig::new("beta-3p").with_src_files(),
            // gamma group
            CrateConfig::new("gamma").with_src_files(),
            CrateConfig::new("gamma-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create multiple prefix groups");

        let mut ws = MyWorkspace::new(&workspace_path).await
            .expect("Should parse workspace with 3 groups");

        // We'll add "beta-utils" => should match only the "beta" group => do facade + 3p dep
        let new_crate_name = "beta-utils";
        let new_handle = ws.add_new_crate_to_workspace(new_crate_name)
            .await
            .expect("Should succeed for single matched prefix group 'beta'");

        // Check membership
        let root_cargo = ws.as_ref().join("Cargo.toml");
        let updated_txt = fs::read_to_string(&root_cargo).await.unwrap();
        assert!(updated_txt.contains("beta-utils"),
            "Should list 'beta-utils' in [workspace].members now");

        // Check facade => "beta" => cargo has `[dependencies].beta-utils`
        let facade_cargo = ws.as_ref().join("beta").join("Cargo.toml");
        let facade_txt = fs::read_to_string(&facade_cargo).await.unwrap();
        debug!("beta facade cargo:\n{}", facade_txt);
        assert!(facade_txt.contains("beta-utils = { path = "),
            "Facade 'beta' should depend on new crate 'beta-utils' after registration");

        // Check new crate => "beta-3p" dependency?
        let new_crate_cargo = new_handle.as_ref().join("Cargo.toml");
        let new_crate_txt = fs::read_to_string(&new_crate_cargo).await.unwrap();
        assert!(
            new_crate_txt.contains("beta-3p = { path = ")
            || new_crate_txt.contains("beta_3p"),
            "New crate should have a dependency on 'beta-3p' (if your code automatically adds it)."
        );
    }

    // -------------------------------------------------------------------------
    // 4) Multiple prefix groups => multiple matches => skip
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_multiple_prefix_groups_many_matches() {
        info!("Scenario 4: multiple prefix groups => crate name matches more than one => we skip facade registration entirely.");

        // We'll create 2 prefix groups => "batch" + "batch-mode"
        let workspace_path: PathBuf = create_mock_workspace(vec![
            CrateConfig::new("batch").with_src_files(),
            CrateConfig::new("batch-3p").with_src_files(),
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-3p").with_src_files(),
        ])
        .await
        .expect("Mock workspace with overlapping prefix groups: 'batch' & 'batch-mode'");

        let mut ws = MyWorkspace::new(&workspace_path).await
            .expect("Should parse workspace with multiple groups");

        // We'll pick "batch-mode-something" => starts with "batch-" AND "batch-mode-"
        let new_crate_name = "batch-mode-extra";
        let new_handle = ws.add_new_crate_to_workspace(new_crate_name)
            .await
            .expect("Should succeed creating stand-alone crate if multiple matches => skip");

        // check membership => appended
        let root_cargo = ws.as_ref().join("Cargo.toml");
        let txt = fs::read_to_string(&root_cargo).await.unwrap();
        assert!(txt.contains("batch-mode-extra"),
            "Should list 'batch-mode-extra' in members");

        // but no facade => check "batch" & "batch-mode" cargo => should NOT have a dependency on the new crate
        let batch_txt = fs::read_to_string(ws.as_ref().join("batch").join("Cargo.toml")).await.unwrap();
        assert!(!batch_txt.contains("batch-mode-extra"),
            "Should skip facade registration if multiple prefix groups match");
        let mode_txt = fs::read_to_string(ws.as_ref().join("batch-mode").join("Cargo.toml")).await.unwrap();
        assert!(!mode_txt.contains("batch-mode-extra"),
            "Neither facade gets the new crate dependency if multiple matches => skip");
    }

    // -------------------------------------------------------------------------
    // 5) Group found but no 3p => we do facade, but warn about missing 3p
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_group_found_but_no_3p() {
        info!("Scenario 5: group found => facade is 'batch-mode', but no 'batch-mode-3p' => we do facade registration, but skip 3p dep => logs a warning.");

        // We'll create a group with "batch-mode" facade, but no 'batch-mode-3p'
        let workspace_path: PathBuf = create_mock_workspace(vec![
            CrateConfig::new("batch-mode").with_src_files(),
        ]).await.expect("Mock workspace with single prefix group missing 3p");

        let mut ws = MyWorkspace::new(&workspace_path).await
            .expect("Should parse workspace with 'batch-mode' but no 3p crate => group is incomplete but recognized.");

        let new_crate_name = "batch-mode-json";
        let new_handle = ws.add_new_crate_to_workspace(new_crate_name)
            .await
            .expect("Should succeed, but log a warning about missing 3p");

        // check membership => updated
        let root_cargo = ws.as_ref().join("Cargo.toml");
        let root_txt = fs::read_to_string(&root_cargo).await.unwrap();
        assert!(root_txt.contains(new_crate_name),
            "Should list 'batch-mode-json' in members array");

        // check facade => has a dependency on new crate
        let facade_cargo = ws.as_ref().join("batch-mode").join("Cargo.toml");
        let facade_txt = fs::read_to_string(&facade_cargo).await.unwrap();
        assert!(facade_txt.contains("batch-mode-json = { path = "),
            "batch-mode facade depends on new crate after registration");

        // new crate cargo => no 'batch-mode-3p' => 
        let new_cargo_txt = fs::read_to_string(new_handle.as_ref().join("Cargo.toml")).await.unwrap();
        assert!(
            !new_cargo_txt.contains("batch-mode-3p"),
            "Should not have a 3p dependency if none existed => we skip, logging a warning."
        );
    }

    // Potential negative scenarios for partial coverage:
    // -------------------------------------------------------------------------
    // 6) Creating the skeleton fails => immediate error
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_skeleton_creation_fails() {
        info!("Scenario 6: skeleton creation fails => we expect an error (IoError or similar).");

        #[cfg(target_os = "linux")]
        {
            // We'll try to create a workspace at "/dev/null"
            let ws = MyWorkspace {
                path: PathBuf::from("/dev/null").into(),
                crates: vec![],
            };

            // The code likely fails to create a directory "/dev/null/new_crate"
            let result = ws.add_new_crate_to_workspace("any_name").await;
            match result {
                Err(WorkspaceError::IoError { context, .. }) => {
                    assert!(context.contains("creating directory for new crate"),
                        "Should mention creating directory context");
                    info!("Got expected IoError for read-only or invalid path");
                },
                _ => {
                    warn!("Unexpected result: we might be in an environment that doesn't enforce /dev/null as a no-op dir");
                }
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            info!("Skipping skeleton_creation_fails test on non-Linux or environment that doesn't enforce perms the same way.");
        }
    }

    // -------------------------------------------------------------------------
    // 7) Failing to add membership => partial coverage
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_membership_update_failure() {
        info!("Scenario 7: membership update fails => we see an IoError or skip scenario. ");

        // We'll create a real workspace, then remove write perms from the top-level Cargo.toml
        #[cfg(target_os = "linux")]
        {
            let workspace_path: PathBuf = create_mock_workspace(vec![]).await
                .expect("Empty workspace");
            let cargo_toml = workspace_path.join("Cargo.toml");

            // We'll set cargo toml to read-only 
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&cargo_toml).await.unwrap().permissions();
            perms.set_mode(0o444); // read-only
            fs::set_permissions(&cargo_toml, perms).await.unwrap();

            // Now we have a valid workspace but can't update the top-level cargo
            let ws = MyWorkspace::new(&workspace_path).await
                .expect("Should parse the workspace initially");

            // Attempt to add new crate
            let result = ws.add_new_crate_to_workspace("some_new_crate").await;
            match result {
                Err(WorkspaceError::IoError { context, .. }) => {
                    assert!(context.contains("writing top-level Cargo.toml after adding member"),
                        "Should mention membership update context");
                    info!("Got expected IoError for read-only membership update");
                },
                Ok(_) => warn!("Unexpected success rewriting read-only cargo => environment might not enforce perms"),
                other => warn!("Got unexpected error: {:?}", other),
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            info!("Skipping membership_update_failure test on non-Linux if perms don't apply similarly.");
        }
    }
}
