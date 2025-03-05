// ---------------- [ File: workspacer-add-new-crate-to-workspace/src/find_single_prefix_match.rs ]
crate::ix!();

/// A small helper that checks if `crate_name` starts with exactly one `prefix + "-"` from the 
/// scanned groups. If we find exactly one match, returns that prefix. If zero or multiple, `None`.
pub fn find_single_prefix_match<P,H>(crate_name: &str, groups: &[PrefixGroup<P,H>]) -> Option<String> 
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Debug + Send + Sync + Clone,
{
    let matching: Vec<_> = groups
        .iter()
        .filter(|g| crate_name.starts_with(&format!("{}-", g.prefix())))
        .collect();

    match matching.len() {
        1 => Some(matching[0].prefix().to_string()),
        0 => {
            debug!("No prefix group matches '{}'", crate_name);
            None
        }
        _ => {
            warn!("Multiple prefix groups could match '{}'; skipping auto registration", crate_name);
            None
        }
    }
}

#[cfg(test)]
mod test_find_single_prefix_match_with_real_data {
    use super::*;

    ///
    /// We demonstrate the use of a *real* workspace scenario to test how `find_single_prefix_match`
    /// behaves after we've formed actual prefix groups from real crates on disk.
    ///
    /// We'll define multiple scenarios by creating a mock workspace with a set of crates
    /// that yield 0, 1, or multiple prefix groups. Then we call `ws.scan()` to retrieve those
    /// groups, and we pass a `crate_name` to `find_single_prefix_match` to see if it finds
    /// exactly one prefix that matches (i.e. `prefix + "-"`), none, or multiple.
    ///
    /// This ensures we're testing with "real" data (i.e., using the scanning logic and real crates)
    /// rather than purely artificial mocks.
    ///

    // -------------------------------------------------------------------------
    // 1) Test: no groups => we have an empty workspace => scanning => 0 prefix groups
    //    => find_single_prefix_match(...) => None
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_no_prefix_groups_returns_none() {
        info!("Scenario 1: no crates => no prefix groups => find_single_prefix_match => None");
        let workspace_path = create_mock_workspace(vec![])
            .await
            .expect("Failed to create empty mock workspace");

        // Build the real workspace
        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Workspace creation should succeed with an empty workspace");

        // We'll scan => expect 0 groups
        let groups = ws.scan().await.expect("Scanning for prefix groups");
        assert_eq!(groups.len(), 0, "No crates => no prefix groups");

        // Now call find_single_prefix_match with some random crate name
        let crate_name = "any-crate";
        let result = find_single_prefix_match::<PathBuf, CrateHandle>(crate_name, &groups);
        assert!(result.is_none(), "Expected None when there are no prefix groups at all");
    }

    // -------------------------------------------------------------------------
    // 2) Test: single prefix group => "batch-mode" with some sub-crate => just 1 group
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_single_prefix_group_match_or_none() {
        info!("Scenario 2: single prefix group => 'batch-mode' + 'batch-mode-3p'");
        let workspace_path = create_mock_workspace(vec![
            // We'll add "batch-mode" => facade
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace with single prefix group");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Workspace creation should succeed with these crates");

        // We'll scan => expect 1 prefix group named "batch-mode"
        let groups = ws.scan().await.expect("Scanning prefix groups");
        assert_eq!(groups.len(), 1, "We should have exactly 1 prefix group: 'batch-mode'");
        assert_eq!(groups[0].prefix(), "batch-mode");

        // 2a) We'll pass a crate name that does NOT start with "batch-mode-"
        let crate_name_no_match = "something_else";
        let result_no_match = find_single_prefix_match::<PathBuf, CrateHandle>(crate_name_no_match, &groups);
        assert!(result_no_match.is_none(),
            "Expected None if crate_name doesn't start with 'batch-mode-'");

        // 2b) We'll pass a crate name that DOES start with "batch-mode-"
        let crate_name_match = "batch-mode-extra";
        let result_match = find_single_prefix_match::<PathBuf, CrateHandle>(crate_name_match, &groups);
        assert_eq!(result_match, Some("batch-mode".to_string()),
            "Expected Some('batch-mode') for an exact single match");
    }

    // -------------------------------------------------------------------------
    // 3) Test: multiple prefix groups => none matches the crate_name
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_multiple_groups_none_matches() {
        info!("Scenario 3: multiple prefix groups => but crate name doesn't match any => None");
        let workspace_path = create_mock_workspace(vec![
            // group 1
            CrateConfig::new("alpha").with_src_files(),
            CrateConfig::new("alpha-3p").with_src_files(),
            // group 2
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace with multiple prefix groups");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Workspace creation should succeed");

        let groups = ws.scan().await.expect("scan() should succeed");
        assert_eq!(groups.len(), 2, "Should have 2 prefix groups: 'alpha' and 'batch-mode'");

        // Our crate_name is "some-unique-crate" => doesn't start with "alpha-" or "batch-mode-"
        let crate_name = "some-unique-crate";
        let result = find_single_prefix_match::<PathBuf, CrateHandle>(crate_name, &groups);
        assert!(result.is_none(),
            "Expected None if crate_name doesn't start with either 'alpha-' or 'batch-mode-'");
    }

    // -------------------------------------------------------------------------
    // 4) Test: multiple prefix groups => exactly one matches
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_multiple_groups_exactly_one_match() {
        info!("Scenario 4: multiple prefix groups => exactly one matches the crate name => returns Some(prefix)");

        let workspace_path = create_mock_workspace(vec![
            // group alpha
            CrateConfig::new("alpha").with_src_files(),
            CrateConfig::new("alpha-3p").with_src_files(),
            // group beta
            CrateConfig::new("beta").with_src_files(),
            CrateConfig::new("beta-3p").with_src_files(),
            // group gamma
            CrateConfig::new("gamma").with_src_files(),
            CrateConfig::new("gamma-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace with multiple prefix groups: alpha, beta, gamma");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Workspace creation should succeed");

        let groups = ws.scan().await.expect("scan() should succeed");
        assert_eq!(groups.len(), 3, "We should have 3 prefix groups: alpha, beta, gamma");

        // We'll pass "beta-something" => only 'beta' group should match
        let crate_name = "beta-something";
        let result = find_single_prefix_match::<PathBuf, CrateHandle>(crate_name, &groups);
        assert_eq!(result, Some("beta".to_string()),
            "Expected Some('beta') for an exact single match among 3 groups");
    }

    // -------------------------------------------------------------------------
    // 5) Test: multiple prefix groups => multiple matches => returns None
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_multiple_groups_many_matches() {
        info!("Scenario 5: multiple prefix groups => multiple matches => returns None");

        // We'll create 2 prefix groups: "batch" and "batch-mode"
        // That means if we have a crate name "batch-mode-something", it might start with both "batch-" and "batch-mode-"
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("batch").with_src_files(),
            CrateConfig::new("batch-3p").with_src_files(),
            CrateConfig::new("batch-mode").with_src_files(),
            CrateConfig::new("batch-mode-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace with overlapping prefix groups: 'batch' and 'batch-mode'");

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace with multiple crates");

        let groups = ws.scan().await.expect("scan() should succeed");
        assert_eq!(groups.len(), 2, "We have 2 prefix groups: 'batch' and 'batch-mode'");

        // We'll pass "batch-mode-extra". That starts with "batch-" AND "batch-mode-".
        let crate_name = "batch-mode-extra";
        let result = find_single_prefix_match::<PathBuf, CrateHandle>(crate_name, &groups);

        // We expect None because it matches 2 groups
        assert!(result.is_none(), 
            "Should return None if multiple prefix groups match the start of 'batch-mode-extra'");
    }
}
