crate::ix!();

// ---------------------- [ File: workspacer-scan-for-prefix-groups/src/lib.rs ] ----------------------

/// Data structure describing one prefix group
///
#[derive(Debug, Clone)]
pub struct PrefixGroup {
    /// The “common prefix” name (e.g. "batch-mode")
    prefix: String,
    /// The facade crate, if any
    prefix_crate: Option<CrateHandle>,
    /// The *-3p crate, if any
    three_p_crate: Option<CrateHandle>,
    /// All crates that belong in this group
    member_crates: Vec<CrateHandle>,
}

/// Trait for scanning the workspace to identify prefix groups.
#[async_trait]
pub trait ScanPrefixGroups {
    /// Returns a list of discovered prefix groups, each with the facade and *-3p crate (if found).
    async fn scan(&self) -> Result<Vec<PrefixGroup>, ScanPrefixGroupsError>;

    /// Optionally validate that each crate is “correctly” referencing the *-3p crate and is reexported by the facade crate, etc.
    async fn validate_prefix_group_cohesion(&self) -> Result<(), ScanPrefixGroupsError>;
}

/// Minimal example impl
pub struct PrefixGroupScanner {
    workspace: Workspace<PathBuf, CrateHandle>, // or a reference
}

impl PrefixGroupScanner {
    pub fn new(workspace: Workspace<PathBuf, CrateHandle>) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl ScanPrefixGroups for PrefixGroupScanner {
    async fn scan(&self) -> Result<Vec<PrefixGroup>, ScanPrefixGroupsError> {
        info!("Scanning workspace for prefix groups...");

        // naive approach: just look at crate names, find “foo-3p”, “foo” (no further hyphens) as facade, etc.
        let mut groups_map = std::collections::HashMap::<String, PrefixGroup>::new();

        for c in self.workspace.crates().iter() {
            let name = c.name();
            if name.ends_with("-3p") {
                let prefix = &name[..name.len()-3];
                let e = groups_map.entry(prefix.to_string()).or_insert_with(|| PrefixGroup {
                    prefix: prefix.to_string(),
                    prefix_crate: None,
                    three_p_crate: None,
                    member_crates: vec![],
                });
                e.three_p_crate = Some(c.clone());
                e.member_crates.push(c.clone());
            } else {
                // if name has exactly one hyphen or zero hyphens, treat it as the “facade crate”
                // or if you prefer other heuristics, do so:
                let dash_count = name.matches('-').count();
                if dash_count <= 1 {
                    let prefix = name.to_string();
                    let e = groups_map.entry(prefix.clone()).or_insert_with(|| PrefixGroup {
                        prefix: prefix.clone(),
                        prefix_crate: None,
                        three_p_crate: None,
                        member_crates: vec![],
                    });
                    e.prefix_crate = Some(c.clone());
                    e.member_crates.push(c.clone());
                } else {
                    // probably belongs to prefix up to second hyphen
                    // your logic for deciding which prefix is the best match
                    // ...
                }
            }
        }

        // finalize
        let results: Vec<PrefixGroup> = groups_map.into_values().collect();
        info!("Discovered {} prefix groups", results.len());
        Ok(results)
    }

    async fn validate_prefix_group_cohesion(&self) -> Result<(), ScanPrefixGroupsError> {
        info!("Validating prefix group cohesion...");
        // Example: for each group, verify that each crate depends on the *-3p crate
        // or is re-exported in the facade, etc.
        Ok(())
    }
}
