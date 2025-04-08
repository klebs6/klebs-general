crate::ix!();

// ------------------------------------------------------------------------------------
// Shared internal layering function for subgraphs, used by the "focus crate" methods
// in both Workspace and CrateHandle. We skip or remove unvisited nodes, apply weighting, etc.
// ------------------------------------------------------------------------------------
pub async fn layered_subgraph_internal(
    graph: &Graph<String, ()>,
    visited: &HashSet<petgraph::prelude::NodeIndex>,
    config: &TopologicalSortConfig
) -> Result<Vec<Vec<String>>, WorkspaceError> {
    let mut layers = Vec::new();
    let mut in_degs = vec![0; graph.node_count()];
    let mut count_subgraph_nodes = 0;

    // Initialize in_degs only for the subgraph
    for idx in graph.node_indices() {
        if visited.contains(&idx) || *config.remove_unwanted_from_graph() {
            // If remove_unwanted_from_graph is true, we assume only visited remain
            count_subgraph_nodes += 1;
            let inc_count = graph
                .neighbors_directed(idx, Direction::Incoming)
                .filter(|pred| visited.contains(pred) || *config.remove_unwanted_from_graph())
                .count();
            in_degs[idx.index()] = inc_count;
        } else {
            // skip node
            in_degs[idx.index()] = usize::MAX;
        }
    }

    let mut remaining = count_subgraph_nodes;
    while remaining > 0 {
        let mut layer_nodes = Vec::new();
        for idx in graph.node_indices() {
            if in_degs[idx.index()] == 0 && !graph[idx].is_empty() {
                layer_nodes.push(idx);
            }
        }
        if layer_nodes.is_empty() {
            error!("layered_subgraph_internal => no zero-in-degree => cycle");
            return Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph { cycle_node_id: 0.into() });
        }

        // sort by weighting or alphabetical
        if let Some(weight_fn) = &config.weighting_fn() {
            layer_nodes.sort_by_key(|&n| weight_fn(&graph[n]));
        } else {
            layer_nodes.sort_by_key(|&n| graph[n].clone());
        }

        let mut layer_strings = Vec::new();
        for &n in &layer_nodes {
            let crate_name = &graph[n];
            if let Some(filter) = &config.filter_fn() {
                if *config.remove_unwanted_from_graph() && !(filter)(crate_name) {
                    continue;
                }
                if !config.remove_unwanted_from_graph() && !visited.contains(&n) {
                    continue;
                }
                if !config.remove_unwanted_from_graph() && !(filter)(crate_name) {
                    continue;
                }
            }
            layer_strings.push(crate_name.clone());
        }
        if !layer_strings.is_empty() {
            layers.push(layer_strings);
        }

        // remove them
        for &n in &layer_nodes {
            let val = &mut in_degs[n.index()];
            if *val != usize::MAX {
                *val = usize::MAX; // mark removed
                remaining -= 1;
                let node_str = &mut graph[n];
                *node_str = "".to_string(); 
                for neigh in graph.neighbors_directed(n, Direction::Outgoing) {
                    if in_degs[neigh.index()] != usize::MAX {
                        in_degs[neigh.index()] = in_degs[neigh.index()].saturating_sub(1);
                    }
                }
            }
        }
    }

    Ok(layers)
}
