crate::ix!();

/// Trait for a simple, flat, topological ordering of *all* crates in the target.
#[async_trait]
pub trait BasicTopologicalSort {
    async fn topological_order_crate_names(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<String>, WorkspaceError>;
}

/// Trait for returning a layered topological ordering of *all* crates in the target.
#[async_trait]
pub trait LayeredTopologicalSort {
    async fn layered_topological_order_crate_names(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<Vec<String>>, WorkspaceError>;
}

/// Trait for focusing on a single crate (or crate handle):
/// produce the subgraph of internal dependencies that lead up to the focus,
/// then return either a flat or layered topological ordering.
#[async_trait]
pub trait FocusCrateTopologicalSort {
    /// Return a flat topological list of all crates leading up to `focus_crate_name`.
    async fn topological_order_upto_crate(
        &self,
        config: &TopologicalSortConfig,
        focus_crate_name: &str
    ) -> Result<Vec<String>, WorkspaceError>;

    /// Return a layered topological grouping of all crates leading up to `focus_crate_name`.
    async fn layered_topological_order_upto_crate(
        &self,
        config: &TopologicalSortConfig,
        focus_crate_name: &str
    ) -> Result<Vec<Vec<String>>, WorkspaceError>;
}

/// Similar trait but specifically for calling on a single `CrateHandle`.
/// If we want to do "topological_sort_internal_deps", we interpret `self` as the "focus crate".
#[async_trait]
pub trait TopologicalSortInternalDeps {
    async fn topological_sort_internal_deps(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<String>, WorkspaceError>;

    async fn layered_topological_order_upto_self(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<Vec<String>>, WorkspaceError>;
}
