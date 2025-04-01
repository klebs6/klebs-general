crate::ix!();

/// Trait for "workspace-like" objects that contain multiple crates. We prune each crate's categories/keywords
/// by calling its `prune_invalid_category_slugs()` method.  
#[async_trait]
pub trait PruneInvalidCategorySlugsFromSubstructures {
    type Error;

    /// Prunes categories/keywords in each of this workspace's member crates.
    /// Returns total count removed across all members.
    async fn prune_invalid_category_slugs_from_members(&mut self) -> Result<usize, Self::Error>;
}

#[async_trait]
impl<P, H, W> PruneInvalidCategorySlugsFromSubstructures for W
where
    // P must meet the same bounds as in WorkspaceInterface:
    P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static,

    // H is the crate type stored in the workspace. We need it to also implement
    // our single-crate pruning trait. 
    H: CrateHandleInterface<P> + PruneInvalidCategorySlugs<Error = CrateError> + Send + Sync + 'static,

    // W is the "workspace-like" type that we can query for crates.
    W: WorkspaceInterface<P, H> + GetCratesMut<P, H> + Send + Sync + 'static,
{
    type Error = WorkspaceError;

    async fn prune_invalid_category_slugs_from_members(&mut self) -> Result<usize, Self::Error> {
        trace!("(Workspace) begin pruning invalid category slugs from substructures");

        let mut total_removed = 0_usize;

        // get mutable access to the vector of Arc<Mutex<H>>:
        let crates_vec = self.crates_mut();
        for crate_arc in crates_vec {
            let mut c_guard = crate_arc.lock().await;
            match c_guard.prune_invalid_category_slugs().await {
                Ok(cnt) => {
                    total_removed += cnt;
                }
                Err(e) => {
                    error!("Error pruning categories in one crate: {:?}", e);
                    // We can fail fast or skip. We'll fail fast:
                    return Err(WorkspaceError::CrateError(e));
                }
            }
        }

        info!("(Workspace) done pruning substructures. total_removed={}", total_removed);
        Ok(total_removed)
    }
}
