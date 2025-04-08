// ---------------- [ File: workspacer-topo/src/config.rs ]
crate::ix!();

/// Configuration for how we do topological sorts.
/// We remove `Debug` because closures cannot derive `Debug` easily.
#[derive(Builder, Getters, Setters, Clone)]
#[builder(setter(into))]
#[getset(get="pub", set="pub")]
pub struct TopologicalSortConfig {
    /// If `true`, reverse the final order (or reverse the layer vector).
    #[builder(default="false")]
    reverse_order: bool,

    /// Optional filter: skip or remove crates that do not satisfy this predicate.
    #[builder(default="None")]
    filter_fn: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,

    /// If `true`, we remove crates that fail the filter from the graph entirely;
    /// if `false`, we keep them in the graph but omit them from final output.
    #[builder(default="false")]
    remove_unwanted_from_graph: bool,

    /// If `true`, we do layered sorting; if `false`, we do flat topological sorting.
    #[builder(default="false")]
    layering_enabled: bool,

    /// If present, used in layered approach to break ties within a layer.
    #[builder(default="None")]
    weighting_fn: Option<Arc<dyn Fn(&str) -> u32 + Send + Sync>>,
}
