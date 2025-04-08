// ---------------- [ File: workspacer-topo/src/traits.rs ]
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

/// Trait for focusing on a single crate/workspace:
/// produce the subgraph of internal dependencies that lead up to the focus,
/// then return either a flat or layered topological ordering.
#[async_trait]
pub trait FocusCrateTopologicalSort {
    async fn topological_order_upto_crate(
        &self,
        config: &TopologicalSortConfig,
        focus_crate_name: &str
    ) -> Result<Vec<String>, WorkspaceError>;

    async fn layered_topological_order_upto_crate(
        &self,
        config: &TopologicalSortConfig,
        focus_crate_name: &str
    ) -> Result<Vec<Vec<String>>, WorkspaceError>;
}

/// Trait specifically for a single `CrateHandle`.
/// "topological_sort_internal_deps" => interpret `self` as the "focus crate".
#[async_trait]
pub trait TopologicalSortInternalDeps {
    /// Returns a flat topological ordering of all internal dependencies leading up to (and including) this crate.
    async fn topological_sort_internal_deps(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<String>, WorkspaceError>;

    /// Returns a layered topological ordering of all internal dependencies up to this crate.
    async fn layered_topological_order_upto_self(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<Vec<String>>, WorkspaceError>;
}
