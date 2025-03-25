// ---------------- [ File: workspacer-scan-for-prefix-groups/src/scan_for_prefix_groups_in_workspace.rs ]
crate::ix!();

#[async_trait]
impl<P, H> ScanPrefixGroups<P, H> for Workspace<P, H>
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
{
    type Error = WorkspaceError;

    ///
    /// Identifies prefix groups using the "longest facade" logic:
    ///
    /// 1) First, we lock each `Arc<AsyncMutex<H>>` to retrieve its name (as `String`) and store
    ///    `(name, arc_handle)` pairs in a local vector.
    /// 2) Sort these pairs by descending name length (so "batch-mode-extra" is processed before "batch-mode").
    /// 3) For each pair `(facade_name, facade_arc)`, if not yet assigned to a group, gather all
    ///    `(other_name, other_arc)` whose names match `facade_name` or start with `"facade_name-"`.
    /// 4) If we get fewer than 2 matches, no group is formed. Otherwise, we form a `PrefixGroup`
    ///    whose `prefix = facade_name`, optionally set `prefix_crate = Some(Arc<H>)` for the one
    ///    whose name is exactly `facade_name`, and `three_p_crate = Some(Arc<H>)` for the one
    ///    whose name is `facade_name-3p`, with the rest in `member_crates`.
    /// 5) We store them as `Arc<H>` (not `Arc<AsyncMutex<H>>`) by locking each handle and cloning
    ///    the inner `H`. This ensures the `PrefixGroup` holds a “snapshot” of each crate handle
    ///    rather than a locked mutex.
    /// 6) Mark all involved crates as assigned. Repeat until all pairs are processed.
    ///
    async fn scan(&self) -> Result<Vec<PrefixGroup<P, H>>, Self::Error> {
        use std::collections::HashSet;
        use tracing::{debug, info, trace};

        info!("Starting prefix group scan with 'longest facade' logic (Arc<AsyncMutex<H>> => Arc<H> snapshot).");

        // Collect (name, arc<mutex<h>>) for each crate by locking them once to get the name.
        let mut ephemeral = Vec::with_capacity(self.crates().len());
        for arc_handle in self.crates().iter().cloned() {
            let arc_handle_clone = arc_handle.clone();
            let locked = arc_handle_clone.lock().await; 
            let name = locked.name().to_string();
            ephemeral.push((name, arc_handle));
        }

        // Sort by descending name length
        ephemeral.sort_by(|(name_a, _), (name_b, _)| name_b.len().cmp(&name_a.len()));

        let mut assigned = HashSet::<String>::new();
        let mut groups = Vec::new();

        // Try forming groups in sorted order
        for (facade_name, facade_arc) in &ephemeral {
            // Already assigned?
            if assigned.contains(facade_name) {
                debug!(
                    "Skipping crate '{}' because it's already assigned to a group",
                    facade_name
                );
                continue;
            }

            // Gather all ephemeral entries that match either exactly facade_name
            // or start with "facade_name-"
            let mut potential = Vec::new();
            for (other_name, other_arc) in &ephemeral {
                if assigned.contains(other_name) {
                    continue;
                }
                if other_name == facade_name
                    || (other_name.starts_with(facade_name)
                        && other_name.get(facade_name.len()..)
                                     .map_or(false, |tail| tail.starts_with('-')))
                {
                    potential.push((other_name.clone(), other_arc.clone()));
                }
            }

            if potential.len() < 2 {
                debug!(
                    "Skipping '{}' because it doesn't have at least 1 additional crate matching '{}-'",
                    facade_name, facade_name
                );
                continue;
            }

            // Mark them all assigned
            for (nm, _) in &potential {
                assigned.insert(nm.clone());
            }

            // Convert potential members into Arc<H> snapshots
            let mut prefix_crate: Option<Arc<H>> = None;
            let mut three_p_crate: Option<Arc<H>> = None;
            let mut member_crates = Vec::new();

            for (nm, arc_mutex_h) in potential {
                let locked = arc_mutex_h.lock().await;
                // Clone the underlying handle into a brand-new Arc<H>
                let handle_snapshot = Arc::new(locked.clone());

                if nm == *facade_name {
                    // facade itself
                    prefix_crate = Some(handle_snapshot);
                } else if nm == format!("{}-3p", facade_name) {
                    // the *-3p crate
                    three_p_crate = Some(handle_snapshot);
                } else {
                    // just a normal group member
                    member_crates.push(handle_snapshot);
                }
            }

            // Build a new PrefixGroup
            let group = PrefixGroupBuilder::default()
                .prefix(facade_name.to_string())
                .prefix_crate(prefix_crate)
                .three_p_crate(three_p_crate)
                .member_crates(member_crates)
                .build()
                .expect("PrefixGroupBuilder should succeed");

            debug!("Formed prefix group with facade='{}'", facade_name);
            groups.push(group);
        }

        // Sort by .prefix() for stable output
        groups.sort_by(|a, b| a.prefix().cmp(&b.prefix()));

        info!("Completed prefix group scan. Found {} groups.", groups.len());
        Ok(groups)
    }
}

#[cfg(test)]
mod test_scan_for_prefix_groups_in_workspace {
    use super::*;

    // -------------------------------------------------------------------------
    // 7C) Test: scan_for_prefix_groups_in_workspace
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_scan_for_prefix_groups_in_workspace() {
        // We'll create a mock workspace with:
        //   - batch-mode
        //   - batch-mode-batch-client
        //   - batch-mode-3p
        //   - random
        //   - random-sub
        // The first 3 should form a group with facade="batch-mode", 3p="batch-mode-3p", members=[batch-mode-batch-client]
        // The last 2 won't form a group unless random-sub matches "random" + dash? Actually it does => facade="random", member="random-sub"

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-batch-client").with_src_files(),
            CrateConfig::new("batch-mode-3p").with_src_files(),
            CrateConfig::new("random").with_src_files(),
            CrateConfig::new("random-sub").with_src_files(),
        ]).await.expect("Failed to create mock workspace");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path).await
            .expect("Should create workspace");

        let groups = ws.scan().await.expect("scan() should succeed");
        // Expect 2 groups
        assert_eq!(groups.len(), 2, "We should form 2 prefix groups: 'batch-mode' and 'random'");

        // 1) find the "batch-mode" group
        let batch_group = groups.iter().find(|g| g.prefix() == "batch-mode")
            .expect("Should find batch-mode group");
        assert!(batch_group.prefix_crate().is_some(), "Should have facade named 'batch-mode'");
        assert_eq!(batch_group.prefix_crate().as_ref().unwrap().name(), "batch-mode");

        let t3p = batch_group.three_p_crate().as_ref()
            .expect("Should have a 3p crate named 'batch-mode-3p'");
        assert_eq!(t3p.name(), "batch-mode-3p");

        let member_names: Vec<_> = batch_group.member_crates().iter().map(|c| c.name().to_string()).collect();
        assert!(member_names.contains(&"batch-mode-batch-client".to_string()));
        assert_eq!(member_names.len(), 1);

        // 2) find the "random" group
        let random_group = groups.iter().find(|g| g.prefix() == "random")
            .expect("Should find random group");
        // There's no random-3p => so no 3p crate
        assert!(random_group.three_p_crate().is_none());
        // facade = "random"
        assert_eq!(random_group.prefix_crate().as_ref().unwrap().name(), "random");
        // member = "random-sub"
        let r_members: Vec<_> = random_group.member_crates().iter().map(|c| c.name().to_string()).collect();
        assert!(r_members.contains(&"random-sub".to_string()));
        assert_eq!(r_members.len(), 1);
    }

    ///
    /// This module provides **exhaustive tests** for the `ScanPrefixGroups` implementation
    /// in `Workspace<P,H>` that uses a **"longest facade"** logic:
    ///
    /// 1) Sort crates descending by name length.
    /// 2) For each crate `X`, if not assigned yet, gather all crates `X` or `X-*`.
    /// 3) If at least 2 are found, form a group with facade = `X`, optional `X-3p` crate,
    ///    and the rest as members.
    /// 4) Mark them assigned, then continue.
    ///
    /// We test various scenarios including empty workspaces, single crates, partial overlaps,
    /// typical multi-crate setups, etc.
    ///

    // -------------------------------------------------------------------------
    // 1) Test: Empty workspace => no groups
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_empty_workspace_no_groups() {
        info!("Starting test_empty_workspace_no_groups");

        // Create a mock workspace with 0 crates
        let workspace_path = create_mock_workspace(vec![])
            .await
            .expect("Failed to create empty mock workspace");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should create an empty workspace");

        let groups = ws.scan().await.expect("scan() should succeed");
        assert_eq!(groups.len(), 0, "No crates => no prefix groups");
    }

    // -------------------------------------------------------------------------
    // 2) Test: Only one crate => no group formed
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_single_crate_no_group() {
        info!("Starting test_single_crate_no_group");

        // Just one crate "foo"
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("foo").with_src_files(),
        ])
        .await
        .expect("Failed to create single-crate workspace");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace with one crate");

        let groups = ws.scan().await.expect("scan() should succeed");
        // Because there's no other crate "foo-*" => no group
        assert_eq!(groups.len(), 0, "One crate alone does not form a prefix group");
    }

    // -------------------------------------------------------------------------
    // 3) Test: Multiple crates, but none have matching prefix => no group
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_multi_crates_no_overlapping_prefix() {
        info!("Starting test_multi_crates_no_overlapping_prefix");

        // We'll create 3 unrelated crates: "alpha", "beta", "charlie"
        // None of them are named "alpha-..." or "beta-...", so no groups
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("alpha").with_src_files(),
            CrateConfig::new("beta").with_src_files(),
            CrateConfig::new("charlie").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace with unrelated crates");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let groups = ws.scan().await.expect("scan() should succeed");
        assert_eq!(groups.len(), 0, "No overlapping prefixes => no groups");
    }

    // -------------------------------------------------------------------------
    // 4) Test: Basic facade + one other => group formed
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_facade_and_one_other() {
        info!("Starting test_facade_and_one_other");

        // We have "foo" and "foo-bar"
        // => "foo" is the facade, "foo-bar" is a member => 1 group
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("foo").with_src_files(),
            CrateConfig::new("foo-bar").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace with two crates");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let groups = ws.scan().await.expect("scan() should succeed");
        assert_eq!(groups.len(), 1, "Should form exactly 1 group");
        let group = &groups[0];
        assert_eq!(group.prefix(), "foo", "Facade crate's name = 'foo'");
        assert!(group.prefix_crate().is_some());
        assert_eq!(group.prefix_crate().as_ref().unwrap().name(), "foo");

        assert!(group.three_p_crate().is_none(), "No foo-3p => none");
        let mems: Vec<_> = group.member_crates().iter().map(|c| c.name()).collect();
        assert_eq!(mems, vec!["foo-bar"], "Member is foo-bar");
    }

    // -------------------------------------------------------------------------
    // 5) Test: Facade + 3p + members
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_facade_with_3p_and_members() {
        info!("Starting test_facade_with_3p_and_members");

        // We'll have "batch-mode", "batch-mode-3p", "batch-mode-engine", "batch-mode-utils"
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-3p").with_src_files(),
            CrateConfig::new("batch-mode-engine").with_src_files(),
            CrateConfig::new("batch-mode-utils").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let groups = ws.scan().await.expect("scan() should succeed");
        assert_eq!(groups.len(), 1, "We should form exactly 1 prefix group: 'batch-mode'");

        let group = &groups[0];
        assert_eq!(group.prefix(), "batch-mode");
        assert!(group.prefix_crate().is_some());
        assert_eq!(group.prefix_crate().as_ref().unwrap().name(), "batch-mode");

        assert!(group.three_p_crate().is_some());
        assert_eq!(group.three_p_crate().as_ref().unwrap().name(), "batch-mode-3p");

        let mem_names: Vec<_> = group.member_crates().iter().map(|c| c.name().to_string()).collect();
        assert_eq!(
            mem_names.len(),
            2,
            "We expect 2 members: batch-mode-engine, batch-mode-utils"
        );
        assert!(mem_names.contains(&"batch-mode-engine".to_string()));
        assert!(mem_names.contains(&"batch-mode-utils".to_string()));
    }

    // -------------------------------------------------------------------------
    // 6) Test: Overlapping prefixes => ensures "longest facade" approach
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_overlapping_prefixes_longest_facade() {
        info!("Starting test_overlapping_prefixes_longest_facade");

        // We have crates:
        //   "batch" (short)
        //   "batch-mode" (longer)
        //   "batch-mode-3p"
        //   "batch-mode-extra"
        //   "batch-other"
        // The logic says we sort by length descending, so "batch-mode-extra" is considered first.
        // Then eventually we see "batch-mode" is a facade, gather "batch-mode-3p" and "batch-mode-extra".
        // "batch-other" might or might not match "batch"? => "batch" sees "batch-other"? => forms a group if 2 exist
        // Actually "batch" can form a group with "batch-other"? => yep => 2 groups in total.

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("batch").with_src_files(),
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-3p").with_src_files(),
            CrateConfig::new("batch-mode-extra").with_src_files(),
            CrateConfig::new("batch-other").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let groups = ws.scan().await.expect("scan() should succeed");
        assert_eq!(groups.len(), 2, "Should form 2 groups: 'batch-mode' and 'batch'");

        // Check the 'batch-mode' group
        let batch_mode_group = groups.iter().find(|g| g.prefix() == "batch-mode")
            .expect("Expected batch-mode group");
        let bm_mems: Vec<_> = batch_mode_group.member_crates().iter().map(|c| c.name().to_string()).collect();
        assert!(batch_mode_group.prefix_crate().is_some());
        assert_eq!(batch_mode_group.prefix_crate().as_ref().unwrap().name(), "batch-mode");
        assert!(batch_mode_group.three_p_crate().is_some(), "Should see batch-mode-3p");
        assert!(bm_mems.contains(&"batch-mode-extra".to_string()));

        // Check the 'batch' group
        let batch_group = groups.iter().find(|g| g.prefix() == "batch")
            .expect("Expected 'batch' group");
        let b_mems: Vec<_> = batch_group.member_crates().iter().map(|c| c.name()).collect();
        assert_eq!(b_mems, vec!["batch-other"], "batch-other is the only member");
    }

    // -------------------------------------------------------------------------
    // 7) Test: We have a crate that is "super-long-name" but it has no `super-long-name-*`,
    //    so it forms no group. Meanwhile, a shorter facade does form a group.
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_no_group_for_super_long_without_suffix() {
        info!("Starting test_no_group_for_super_long_without_suffix");

        // We'll add a crate "super-long-name" with no others that start with that name + '-'
        // We'll also add "abc", "abc-3p"
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("super-long-name").with_src_files(),
            CrateConfig::new("abc").with_src_files(),
            CrateConfig::new("abc-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        let groups = ws.scan().await.expect("scan() should succeed");
        // We expect "super-long-name" not to form a group => only "abc" does
        assert_eq!(groups.len(), 1, "Only 'abc' forms a group with 'abc-3p'");

        let group = &groups[0];
        assert_eq!(group.prefix(), "abc", "Facade is 'abc'");
        assert!(group.prefix_crate().is_some());
        assert!(group.three_p_crate().is_some());
        assert_eq!(group.three_p_crate().as_ref().unwrap().name(), "abc-3p");
    }
}
