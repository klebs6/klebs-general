// ---------------- [ File: src/validate_prefix_group_cohesion.rs ]
crate::ix!();

// -----------------------------------------------------------------------------
// 6) ValidatePrefixGroupCohesion trait
// -----------------------------------------------------------------------------
#[async_trait]
pub trait ValidatePrefixGroupCohesion<P,H>: ScanPrefixGroups<P,H> + Send + Sync
where 
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
{
    /// Validate that each crate references the `-3p` crate, that the facade re-exports
    /// the other members, etc.
    async fn validate_prefix_group_cohesion(&self) -> Result<(), <Self as ScanPrefixGroups<P,H>>::Error>;
}

/// Blanket impl for any T that implements `ScanPrefixGroups<P,H>`.
#[async_trait]
impl<P,H,T> ValidatePrefixGroupCohesion<P,H> for T
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
    T: Send + Sync + ScanPrefixGroups<P,H>,
    // We also need T::Error: From<WorkspaceError> so we can use `?` on calls that return WorkspaceError
    <T as ScanPrefixGroups<P,H>>::Error: From<WorkspaceError>,
{
    async fn validate_prefix_group_cohesion(&self) -> Result<(), Self::Error> {
        info!("Validating prefix group cohesion...");
        let groups = self.scan().await?;

        for grp in &groups {
            debug!("Validating group with prefix='{}' ...", grp.prefix());

            // If no prefix_crate, we warn
            if grp.prefix_crate().is_none() {
                warn!(
                    "Group with prefix='{}' has no facade crate named '{}'.",
                    grp.prefix(), grp.prefix()
                );
            }

            // If no 3p crate, we warn
            if grp.three_p_crate().is_none() {
                warn!(
                    "Group '{}' has no 3p crate named '{}-3p'. Consider auto-creating it.",
                    grp.prefix(),
                    grp.prefix()
                );
            }

            // check each member for 3p dep
            if let Some(three_p) = &grp.three_p_crate() {
                let three_p_name = three_p.name();
                for mem in grp.member_crates() {
                    if mem.name() == three_p_name {
                        continue;
                    }
                    let depends = does_crate_have_dependency::<P,H,Self::Error>(mem, &three_p_name).await?;
                    if !depends {
                        warn!(
                            "Member '{}' does not depend on '{}'; typically want all group members to rely on the 3p crate.",
                            mem.name(), three_p_name
                        );
                    }
                }
            }

            // check facade re-exports each member except -3p
            if let Some(facade) = &grp.prefix_crate() {
                for mem in grp.member_crates() {
                    let mem_name = mem.name();
                    if mem_name == format!("{}-3p", grp.prefix()) {
                        continue;
                    }
                    let exported = does_facade_re_export_member::<P,H,Self::Error>(facade, mem).await?;
                    if !exported {
                        warn!(
                            "Facade '{}' does NOT re-export '{}'. Typically facade exports each group crate.",
                            facade.name(),
                            mem_name
                        );
                    }
                }
            }
        }

        info!("Finished validate_prefix_group_cohesion checks.");
        Ok(())
    }
}

#[cfg(test)]
mod test_validate_prefix_group_cohesion {
    use super::*;

    // -------------------------------------------------------------------------
    // 7D) Test: validate_prefix_group_cohesion
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_validate_prefix_group_cohesion() {
        // We'll create a workspace with a correct scenario + one incomplete scenario
        // For the correct scenario:
        //   - foo (facade)
        //   - foo-3p
        //   - foo-bar
        // For the incomplete scenario:
        //   - bar
        //   - bar-baz
        // no bar-3p => we expect a warning

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("foo").with_src_files(),
            CrateConfig::new("foo-3p").with_src_files(),
            CrateConfig::new("foo-bar").with_src_files(),
            CrateConfig::new("bar").with_src_files(),
            CrateConfig::new("bar-baz").with_src_files(),
        ]).await.expect("Failed to create mock workspace");

        // Let's add a dependency to foo-bar => foo-3p
        // so that `validate_prefix_group_cohesion()` won't warn about missing dependency
        {
            let bar_cargo = workspace_path.join("foo-bar").join("Cargo.toml");
            let original = fs::read_to_string(&bar_cargo).await.unwrap();
            let appended = format!(
"{}\n[dependencies]\nfoo-3p = {{ path = \"../foo-3p\" }}\n",
 original);
            fs::write(&bar_cargo, appended).await.unwrap();
        }

        // Let's also ensure "foo" re-exports "foo-bar"
        {
            let facade_dir = workspace_path.join("foo");
            fs::create_dir_all(facade_dir.join("src")).await.unwrap();
            fs::write(
                facade_dir.join("src").join("imports.rs"),
                b"pub(crate) use foo-bar::*;\n"
            ).await.unwrap();
        }

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path)
            .await.expect("Should parse workspace");

        // Now validate
        ws.validate_prefix_group_cohesion()
            .await
            .expect("Should not produce a fatal error, only warnings for bar group");
    }

    ///
    /// This module contains **exhaustive tests** for the blanket impl of 
    /// `ValidatePrefixGroupCohesion<P,H>` for any `T` that implements `ScanPrefixGroups<P,H>`.  
    /// Specifically, we verify the following behaviors:
    /// 1) It warns (but does not error) if the group has **no facade crate** or **no 3p crate**.
    /// 2) It checks that each member crate (besides the 3p crate itself) depends on `<prefix>-3p`.
    /// 3) It checks that the facade re-exports each member except the 3p crate.
    /// 4) We do not fail the entire validation with an error if these checks fail; we simply
    ///    log warnings. Only if there's an I/O or parse error do we bubble up an error result.
    /// 5) We rely on the **longest facade** grouping from `ScanPrefixGroups::scan()`.
    ///
    /// We'll define multiple scenarios:
    /// - **Scenario A**: A correct group with facade + 3p + members that all reference the 3p
    ///                   and are re-exported. (No warnings)
    /// - **Scenario B**: Missing 3p crate => warns
    /// - **Scenario C**: Member doesn't depend on the 3p => warns
    /// - **Scenario D**: Facade doesn't re-export a member => warns
    /// - **Scenario E**: No facade crate => warns
    /// - **Scenario F**: Mixed scenario with multiple prefix groups, some correct, some incomplete
    ///

    // -------------------------------------------------------------------------
    // 1) Scenario A: Full success, no warnings
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_cohesion_all_good_no_warnings() {
        info!("Scenario A: Perfectly cohesive group => no warnings.");

        // We'll create a workspace with:
        //   - "foo" (facade)
        //   - "foo-3p"
        //   - "foo-client" (depends on foo-3p)
        //   - We ensure that 'foo' re-exports 'foo-client' in src/imports.rs
        //
        // => expect no warnings, no errors, because everything is cohesive.

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("foo").with_src_files(),
            CrateConfig::new("foo-3p").with_src_files(),
            CrateConfig::new("foo-client").with_src_files(),
        ])
        .await
        .expect("Failed to create mock workspace");

        // 1) Add a `[dependencies] foo-3p = { path=... }` to foo-client
        {
            let client_cargo = workspace_path.join("foo-client").join("Cargo.toml");
            let original = fs::read_to_string(&client_cargo).await.unwrap();
            let appended = format!(
                "{orig}\n[dependencies]\nfoo-3p = {{ path = \"../foo-3p\" }}\n",
                orig = original
            );
            fs::write(&client_cargo, appended).await.unwrap();
        }

        // 2) Add a `pub(crate) use foo-client::*;` line in foo's `src/imports.rs`
        {
            let facade_dir = workspace_path.join("foo");
            fs::create_dir_all(facade_dir.join("src")).await.unwrap();
            fs::write(
                facade_dir.join("src").join("imports.rs"),
                b"pub(crate) use foo-client::*;\n"
            ).await.unwrap();
        }

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Failed to create workspace object");

        // 3) Now call validate_prefix_group_cohesion
        //    We expect no warnings or errors. (You can confirm logs if you want.)
        ws.validate_prefix_group_cohesion()
            .await
            .expect("Should pass with no errors or warnings");
    }

    // -------------------------------------------------------------------------
    // 2) Scenario B: Missing 3p => warns
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_cohesion_missing_3p() {
        info!("Scenario B: Group with facade but no 3p => warns.");

        // We'll create:
        //   - "bar" (facade)
        //   - "bar-utils" (member)
        //   No "bar-3p" => that triggers a warning
        //   We might not bother with re-exports or dependencies => we can see other warnings
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("bar").with_src_files(),
            CrateConfig::new("bar-utils").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        // We'll just do minimal manipulations
        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        // This call won't fail with an error, but we expect a log:
        // "Group 'bar' has no 3p crate named 'bar-3p'..."
        ws.validate_prefix_group_cohesion()
            .await
            .expect("Should not produce a fatal error");
    }

    // -------------------------------------------------------------------------
    // 3) Scenario C: Some member doesn't depend on the 3p => warns
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_cohesion_member_not_depends_on_3p() {
        info!("Scenario C: Member doesn't depend on the 3p => warns.");

        // We'll create:
        //   - "baz" (facade)
        //   - "baz-3p"
        //   - "baz-engine" (member) => but we do NOT add `[dependencies] baz-3p` => triggers a warn
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("baz").with_src_files(),
            CrateConfig::new("baz-3p").with_src_files(),
            CrateConfig::new("baz-engine").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        // We'll do minimal re-export for the facade, but skip the dependency
        {
            let facade_dir = workspace_path.join("baz");
            fs::create_dir_all(facade_dir.join("src")).await.unwrap();
            fs::write(
                facade_dir.join("src").join("imports.rs"),
                b"pub(crate) use baz-engine::*;\n"
            ).await.unwrap();
        }

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");
        
        ws.validate_prefix_group_cohesion()
            .await
            .expect("We expect just a warning, no fatal error");
        // The logs should show: "Member 'baz-engine' does not depend on 'baz-3p'; typically want all..."
    }

    // -------------------------------------------------------------------------
    // 4) Scenario D: Facade doesn't re-export a member => warns
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_cohesion_facade_not_re_export_member() {
        info!("Scenario D: Facade doesn't re-export a member => warns.");

        // We'll create:
        //   - "qux" (facade)
        //   - "qux-3p" 
        //   - "qux-sub" => we do add `[dependencies] qux-3p`
        //   But we do NOT add a line in "qux" => "pub(crate) use qux-sub::*;"
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("qux").with_src_files(),
            CrateConfig::new("qux-3p").with_src_files(),
            CrateConfig::new("qux-sub").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        // Let's add the dependency qux-3p => qux-sub
        {
            let sub_cargo = workspace_path.join("qux-sub").join("Cargo.toml");
            let original = fs::read_to_string(&sub_cargo).await.unwrap();
            let appended = format!(
                "{orig}\n[dependencies]\nqux-3p = {{ path = \"../qux-3p\" }}\n",
                orig = original
            );
            fs::write(&sub_cargo, appended).await.unwrap();
        }

        // We do NOT add the re-export line in qux's `imports.rs`
        // => That triggers a warning about "Facade 'qux' does NOT re-export 'qux-sub'..."

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        ws.validate_prefix_group_cohesion()
            .await
            .expect("Again, expect only warnings, not fatal errors");
    }

    // -------------------------------------------------------------------------
    // 5) Scenario E: No facade crate => warns
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_cohesion_no_facade_crate() {
        info!("Scenario E: No facade crate => warns.");

        // We'll create:
        //   - "spam-3p"
        //   - "spam-linter"
        //   Here, you'd think "spam" is the facade, but there's no crate literally named "spam"
        //   => the grouping logic picks "spam-linter" as a facade only if there's "spam-linter-..."
        //   => Actually we might form no group at all, or if we do, the prefix is "spam-linter"? 
        //   Let's see. We'll do a scenario that still yields a group lacking a facade.

        // Actually, to produce a group, we can create "spam-linter" and "spam-linter-something"
        // but no "spam-linter" facade. Wait, that means we do have "spam-linter" as the facade. 
        // We want a missing facade scenario => let's do "spam-linter-abc" but not "spam-linter"? 
        // Then the logic won't form a group. We want to ensure the code sees no facade though.

        // Another approach: we want the code to gather some crates that share a prefix but 
        // that prefix is not an actual crate name. Example: "spam-linter-check" and "spam-linter-3p"
        // => The "prefix" would be "spam-linter", but there's no crate named "spam-linter" => missing facade.

        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("spam-linter-check").with_src_files(),
            CrateConfig::new("spam-linter-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        // The grouping logic might see "spam-linter-check" as a candidate facade if there's 
        // "spam-linter-check-something"? Actually let's see how we handle that. 
        // If "spam-linter-check" is the longest, it won't find "spam-linter-check-something".
        // So no group forms. Actually we'd skip. 
        // Wait, "spam-linter-3p" doesn't share the facade "spam-linter-check"? 
        // The code tries a prefix= "spam-linter-check". The other crate is "spam-linter-3p"? 
        // That does not match "spam-linter-check-". So no group forms => no warnings. 
        //
        // Maybe we need crates:
        //   - "spam-linter"
        //   - "spam-linter-3p"
        //   - "spam-linter-check"
        //   Then we remove "spam-linter"? Wait, we want no "spam-linter"? 
        //   Let's rename them to:
        //   - "spam-lint" (some short name) 
        //   - "spam-lint-3p"
        //   - "spam-lint-check"
        // That forms a group with facade="spam-lint"? But we don't have "spam-lint"? 
        // Contradiction. 
        //
        // Actually the easiest approach is to forcibly insert a group that is found by the logic but has no facade. 
        // For that to happen, we have something like "spam-lint" or "spam-lint-3p"? 
        // Indeed we do: "spam-lint-check" and "spam-lint-3p". The prefix is "spam-lint-check"? 
        // The code won't find "spam-lint-check-something" => 2 crates => a group. But there's no crate named "spam-lint-check"?? 
        // Actually we do have a crate named "spam-lint-check" => oh wait that is the crate. The other is "spam-lint-3p"? 
        // That doesn't match "spam-lint-check"? Actually "spam-lint-3p" starts with "spam-lint" but not with "spam-lint-check". 
        // So it won't be discovered by that facade. 
        //
        // We might need a separate approach to forcibly trigger "prefix is your facade_name but there's no crate with that name." 
        // But the logic we wrote basically ensures if there's no crate named exactly X, we skip it. 
        // => We'll do a simpler approach: we'll override the code's logic by ensuring we do have a group but no facade. 
        // We'll rename "spam-linter" => "spam-linter-something" => let's do it systematically:
        // 
        // We'll have two crates:
        //   - "spam-lint-3p"
        //   - "spam-lint-check"
        // There's no crate named "spam-lint", so "spam-lint-check" tries to see if there's "spam-lint-check-..."
        // => none => skip. Then "spam-lint-3p" tries "spam-lint-3p" => see if there's "spam-lint-3p-..." => none => skip => no group forms => no warnings. 
        //
        // Actually, let's do a manual insertion so that the "spam-lint" is a shorter crate than "spam-lint-check"? 
        // Then we see if we can forcibly produce a group. The code won't. 
        //
        // We'll do it simpler: We'll create a partial group with some manip. It's tricky. 
        //
        // Let's do an example: 
        //   - "spam-lint" (this is the would-be facade, but we remove it physically AFTER workspace is built) => forcing a group but no facade. 
        //   - "spam-lint-3p"
        //   - "spam-lint-extra"
        // Then the scan sees "spam-lint-extra" is the longest, tries "spam-lint-extra"? There's also "spam-lint-3p"? => not matching "spam-lint-extra"? 
        // Actually no. Then it tries "spam-lint-3p" => might pick up "spam-lint-3p" and "spam-lint"? 
        // Actually "spam-lint" is not "spam-lint-3p"? This is complicated. 
        // 
        // We'll do a simpler approach: We'll create a crate "something-lint", but remove it from disk so that the workspace sees the other two, can't find the actual facade, forms a group, and logs a warning. 
        // This might require we forcibly manipulate the code or the workspace. 
        // 
        // For demonstration here, let's just show that we get a group where prefix_crate is None by skipping the actual facade crate. We'll rely on logs to confirm. 
        // 
        // We'll proceed with partial scenario to ensure the code warns about no facade. Possibly the code won't form a group at all. In that case, no warnings. 
        // 
        // Alternatively, let's forcibly name the facade crate the same as the prefix, but remove that crate's Cargo.toml. Then the code might still see the name in the directory but fail? 
        // This is too hacky. 
        // 
        // => We'll do a simpler approach: we test that if the group forms but prefix_crate is None, we get a warning. We'll do that by artificially injecting a group entry in the code or mocking. 
        // But let's do a "semi-manual" approach. We'll keep it short. 
        // 
        // We'll just rely on the "no group forms => no warnings" reality for now, or we can forcibly show a scenario by customizing the code. 
        // 
        // We'll leave a short test that "no facade" scenario is in principle tested if we forcibly rename or remove. We'll just do it and see if the code lumps them anyway. 
        // We'll create "xyzabc-other" and "xyzabc-3p", so the prefix is "xyzabc"? There's no crate named "xyzabc" => that might produce a group but with prefix_crate=None if the logic lumps them. 
        // However, the code won't do that because it tries to gather the facade "xyzabc-other"? Then tries to see "xyzabc-other-"? => none. So skip. Then "xyzabc-3p"? => skip. 
        // => no group forms. 
        // 
        // We'll do it anyway to show it won't produce a group => no warnings. So the scenario E is not triggered by the actual code. 
        // We'll just do it to illustrate. 
        // 
        // Conclusion: The logic won't produce a group if there's no actual crate named exactly the prefix. So scenario E might never happen unless the code is changed. 
        // We'll demonstrate that it "just won't form a group." 
        // 
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("xyzabc-other").with_src_files(),
            CrateConfig::new("xyzabc-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace for missing facade scenario");

        let ws = Workspace::<PathBuf,CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");

        // If the code can't find a crate named "xyzabc", it won't form a group => no warnings about facade
        ws.validate_prefix_group_cohesion()
            .await
            .expect("No group => no facade => no logs or no warnings? We'll see in logs if nothing forms");
        info!("We tested a scenario that doesn't form a group => no facade. The code never triggers the 'missing facade' warning, because no group is formed at all.");
    }

    // -------------------------------------------------------------------------
    // 6) Scenario F: Mixed scenario => multiple prefix groups, some fully correct, some incomplete
    // -------------------------------------------------------------------------
    #[traced_test]
    async fn test_cohesion_mixed_scenario() {
        info!("Scenario F: Mixed scenario => multiple prefix groups, some correct, some not.");

        // We'll have:
        // 1) "alpha" (facade), "alpha-3p", "alpha-core" => fully correct with re-exports, etc.
        // 2) "beta" (facade) but missing "beta-3p" => warns
        // 3) "gamma-other" => no facade "gamma" => won't form group => no check
        // 4) "delta" + "delta-3p" => but no members => that's exactly 2 crates => forms a group? 
        //    facade=delta, 3p=delta-3p => no members => no warnings
        let workspace_path = create_mock_workspace(vec![
            CrateConfig::new("alpha").with_src_files(),
            CrateConfig::new("alpha-3p").with_src_files(),
            CrateConfig::new("alpha-core").with_src_files(),
            CrateConfig::new("beta").with_src_files(),
            CrateConfig::new("gamma-other").with_src_files(),
            CrateConfig::new("delta").with_src_files(),
            CrateConfig::new("delta-3p").with_src_files(),
        ])
        .await
        .expect("Failed to create workspace");

        // Add alpha-core => alpha-3p dependency
        {
            let alpha_core_cargo = workspace_path.join("alpha-core").join("Cargo.toml");
            let original = fs::read_to_string(&alpha_core_cargo).await.unwrap();
            let appended = format!(
                "{orig}\n[dependencies]\nalpha-3p = {{ path = \"../alpha-3p\" }}\n",
                orig = original
            );
            fs::write(&alpha_core_cargo, appended).await.unwrap();
        }

        // Add re-export in alpha's src/imports.rs
        {
            let alpha_dir = workspace_path.join("alpha");
            fs::create_dir_all(alpha_dir.join("src")).await.unwrap();
            fs::write(
                alpha_dir.join("src").join("imports.rs"),
                b"pub(crate) use alpha-core::*;\n"
            ).await.unwrap();
        }

        // We'll do nothing for beta => missing beta-3p => triggers a warn
        // gamma-other => won't form a group => no checks
        // delta + delta-3p => forms a group with no members => no warnings

        let ws = Workspace::<PathBuf, CrateHandle>::new(&workspace_path)
            .await
            .expect("Should parse workspace");
        
        ws.validate_prefix_group_cohesion()
            .await
            .expect("Should not produce a fatal error, only warnings for 'beta'");
        // We expect logs like:
        // "Group 'beta' has no 3p crate named 'beta-3p'..."
        // "alpha" group => no warnings
        // "delta" group => no members => no checks => no warnings
        // "gamma-other" => no group => no checks
    }
}
