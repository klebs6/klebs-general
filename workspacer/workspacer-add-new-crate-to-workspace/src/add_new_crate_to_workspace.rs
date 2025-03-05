// ---------------- [ File: src/add_new_crate_to_workspace.rs ]
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
        &self,
        new_crate_name: &str,
    ) -> Result<H, Self::Error>;
}

/// Blanket impl for any T that implements `ScanPrefixGroups`, `RegisterInPrefixGroup`, etc.
#[async_trait]
impl<P,H,T> AddNewCrateToWorkspace<P,H> for T
where
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Debug + Send + Sync + Clone,
    T: ScanPrefixGroups<P,H, Error = WorkspaceError>
        + RegisterInPrefixGroup<P,H,Error = WorkspaceError>
        + CreateCrateSkeleton<P>
        + AddInternalDependency<P,H,Error = WorkspaceError> // if we want to add a new crate's own dep on prefix-3p
        + AddToWorkspaceMembers<P>
        + WorkspaceInterface<P,H>  // so we can handle workspace members
        + AsRef<Path>              // treat T as path
        + Sync
        + Send,
{
    type Error = WorkspaceError;

    async fn add_new_crate_to_workspace(
        &self,
        new_crate_name: &str,
    ) -> Result<H, Self::Error> {
        info!("add_new_crate_to_workspace('{}') - start", new_crate_name);

        // 1) Create the crate on disk
        let new_crate_dir = self.create_new_crate_skeleton(new_crate_name).await?;

        // 2) Add to the top-level `[workspace].members` if necessary
        self.add_to_workspace_members(&new_crate_dir).await?;

        // 3) Build a new CrateHandle
        let new_handle = H::new(&new_crate_dir).await.map_err(|e| WorkspaceError::CrateError(e))?;
        debug!("Created handle for crate='{}'", new_handle.name());

        // 4) Try to detect a prefix group by scanning
        let groups = self.scan().await?;
        let maybe_prefix = find_single_prefix_match(&new_handle.name(), &groups);

        if let Some(prefix) = maybe_prefix {
            // find the group
            if let Some(grp) = groups.iter().find(|g| *g.prefix() == prefix) {
                // a) register the new crate in the prefix crate's facade
                if let Some(facade_cr) = grp.prefix_crate() {
                    self.register_in_prefix_crate(facade_cr, &new_handle).await?;
                }

                // b) optionally ensure new crate depends on <prefix>-3p if that 3p crate exists
                if let Some(three_p) = grp.three_p_crate() {
                    // We can do a "new_handle depends on <prefix>-3p" if you want
                    self.add_internal_dependency(&new_handle, three_p).await?;
                } else {
                    warn!("Group '{}' has no 3p crate => cannot add a dependency from new crate to the 3p crate", prefix);
                }
            } else {
                warn!("No group data found for prefix='{}' even though we had a match? Possibly a concurrency issue? Skipping facade registration.", prefix);
            }
        } else {
            debug!("No single matching prefix => new crate is stand-alone. Done.");
        }

        Ok(new_handle)
    }
}
