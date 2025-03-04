// ---------------- [ File: src/scan_for_prefix_groups.rs ]
crate::ix!();

// -----------------------------------------------------------------------------
// 2) Trait for scanning the workspace to identify prefix groups.
// -----------------------------------------------------------------------------
#[async_trait]
pub trait ScanPrefixGroups<P,H> 
where 
    for<'async_trait> P: Debug + Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync + Debug + Clone,
{
    type Error;

    /// Returns a list of discovered prefix groups, each with the facade crate and *-3p crate (if found).
    async fn scan(&self) -> Result<Vec<PrefixGroup<P,H>>, Self::Error>;
}
