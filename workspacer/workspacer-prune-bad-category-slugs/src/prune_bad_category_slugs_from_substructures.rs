crate::ix!();

#[async_trait]
pub trait PruneInvalidCategorySlugsFromSubstructures {
    type Error;

    /// Prunes category slugs in each of this workspace’s crates. Returns total removed across all members.
    async fn prune_invalid_category_slugs_from_members(&mut self) -> Result<usize, Self::Error>;
}

#[async_trait]
impl<P,H> PruneInvalidCategorySlugsFromSubstructures for Workspace<P,H>
where
    // 1) The same `<P>` constraints from the `WorkspaceInterface` definitions:
    P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static,

    // 2) The crate type `H` must be a valid crate handle that also implements single-crate pruning:
    H: CrateHandleInterface<P> 
       + PruneInvalidCategorySlugs<Error = CrateError> 
       + Send
       + Sync
       + 'static,
{
    type Error = WorkspaceError;

    async fn prune_invalid_category_slugs_from_members(&mut self) -> Result<usize, Self::Error> {
        trace!("(Workspace) begin pruning substructures");
        let mut total_removed = 0_usize;

        let crates_vec = self.crates_mut();
        for crate_arc in crates_vec {
            let mut crate_guard = crate_arc.lock().await;
            match crate_guard.prune_invalid_category_slugs().await {
                Ok(cnt) => {
                    total_removed += cnt;
                }
                Err(e) => {
                    error!("Error pruning categories in crate: {:?}", e);
                    return Err(WorkspaceError::CrateError(e));
                }
            }
        }

        info!("(Workspace) done pruning substructures. total_removed={}", total_removed);
        Ok(total_removed)
    }
}
