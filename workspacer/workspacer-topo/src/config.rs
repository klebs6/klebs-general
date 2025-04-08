crate::ix!();

/// Configuration for how we do topological sorts.
#[derive(Builder, Getters, Setters, Debug, Clone)]
#[builder(setter(into))]
#[getset(get = "pub", set = "pub")]
pub struct TopologicalSortConfig {
    /// If `true`, we reverse the final order (or reverse the layer vector).
    reverse_order: bool,

    /// Optional filter: skip or remove crates that do not satisfy this predicate.
    filter_fn: Option<Arc<dyn Fn(&str) -> bool + Send + Sync>>,

    /// If `true`, we remove crates that fail the filter from the graph entirely;
    /// if `false`, we keep them in the graph but omit them from final output.
    remove_unwanted_from_graph: bool,

    /// If `true`, we do layered sorting; if `false`, we do flat topological sorting.
    layering_enabled: bool,

    /// If present, used in layered approach to break ties within a layer.
    weighting_fn: Option<Arc<dyn Fn(&str) -> u32 + Send + Sync>>,
}
