crate::ix!();

// ----------------------------------------------------------------------------------
// FocusCrateTopologicalSortForCrateHandle => implement on CrateHandle
// We'll do the same idea, but the "focus crate" is `self.name()`.
// We must look up or build the workspace or a graph. Possibly the CrateHandle can do
// "CrateHandle -> generate a mini-graph of itself + transitive dependencies" if that is known.
//
// If you have a direct approach to get the entire workspace graph from a single CrateHandle,
// you can replicate the same logic as above. Or you might require that the user pass a
// reference to a workspace. This example shows a direct approach if your code can do it.
// ----------------------------------------------------------------------------------
#[async_trait]
impl TopologicalSortInternalDeps for CrateHandle {
    async fn topological_sort_internal_deps(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<String>, WorkspaceError> {
        trace!("CrateHandle::topological_sort_internal_deps => focus='{}'", self.name());
        // If your CrateHandle can build the entire workspace graph of all crates, do that.
        // Or if there's a separate method "self.generate_dependency_graph_of_my_workspace()" we call it.
        // Below is pseudocode:
        let workspace_graph = self.generate_my_workspace_graph().await?; // adapt to your real code
        let my_name = self.name().to_string();

        // The rest is the same logic as "topological_order_upto_crate" in a workspace.
        let mut focus_idx = None;
        for idx in workspace_graph.node_indices() {
            if workspace_graph[idx] == my_name {
                focus_idx = Some(idx);
                break;
            }
        }
        if focus_idx.is_none() {
            warn!("Crate '{}' not found in the graph => empty", my_name);
            return Ok(vec![]);
        }
        let focus_idx = focus_idx.unwrap();

        // gather ancestors
        let mut visited = HashSet::new();
        visited.insert(focus_idx);
        let mut stack = vec![focus_idx];
        while let Some(cur) = stack.pop() {
            for pred in workspace_graph.neighbors_directed(cur, Direction::Incoming) {
                if !visited.contains(&pred) {
                    visited.insert(pred);
                    stack.push(pred);
                }
            }
        }

        if config.remove_unwanted_from_graph() {
            let mut to_remove = Vec::new();
            for idx in workspace_graph.node_indices() {
                if !visited.contains(&idx) {
                    to_remove.push(idx);
                }
            }
            to_remove.sort_by_key(|i| i.index());
            to_remove.reverse();
            for idx in to_remove {
                workspace_graph.remove_node(idx);
            }
        }

        // layering or non-layering
        if config.layering_enabled() {
            let layers = layered_subgraph_internal(&workspace_graph, &visited, config).await?;
            // flatten
            let mut flat = Vec::new();
            for layer in layers {
                for crate_name in layer {
                    flat.push(crate_name);
                }
            }
            return Ok(flat);
        } else {
            // standard toposort
            let sorted_idx = match toposort(&workspace_graph, None) {
                Ok(list) => list,
                Err(Cycle{node_id}) => {
                    error!("Cycle in subgraph => node={:?}", node_id);
                    return Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                        cycle_node_id: node_id,
                    });
                }
            };
            let mut result = Vec::new();
            for idx in sorted_idx {
                if !config.remove_unwanted_from_graph() && !visited.contains(&idx) {
                    continue;
                }
                // filter
                if let Some(filter) = &config.filter_fn() {
                    let name = &workspace_graph[idx];
                    if config.remove_unwanted_from_graph() && !(filter)(name) {
                        continue;
                    }
                    if !config.remove_unwanted_from_graph() && !(filter)(name) {
                        continue;
                    }
                }
                result.push(workspace_graph[idx].clone());
            }
            if config.reverse_order() {
                result.reverse();
            }
            return Ok(result);
        }
    }

    async fn layered_topological_order_upto_self(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<Vec<String>>, WorkspaceError> {
        trace!("CrateHandle::layered_topological_order_upto_self => '{}'", self.name());
        let workspace_graph = self.generate_my_workspace_graph().await?; // adapt to real code
        let my_name = self.name().to_string();

        let mut focus_idx = None;
        for idx in workspace_graph.node_indices() {
            if workspace_graph[idx] == my_name {
                focus_idx = Some(idx);
                break;
            }
        }
        if focus_idx.is_none() {
            warn!("Crate '{}' not found => empty layering", my_name);
            return Ok(vec![]);
        }
        let focus_idx = focus_idx.unwrap();

        // gather ancestors
        let mut visited = HashSet::new();
        visited.insert(focus_idx);
        let mut stack = vec![focus_idx];
        while let Some(cur) = stack.pop() {
            for pred in workspace_graph.neighbors_directed(cur, Direction::Incoming) {
                if !visited.contains(&pred) {
                    visited.insert(pred);
                    stack.push(pred);
                }
            }
        }

        if config.remove_unwanted_from_graph() {
            let mut to_remove = Vec::new();
            for idx in workspace_graph.node_indices() {
                if !visited.contains(&idx) {
                    to_remove.push(idx);
                }
            }
            to_remove.sort_by_key(|i| i.index());
            to_remove.reverse();
            for idx in to_remove {
                workspace_graph.remove_node(idx);
            }
        }

        let layers = layered_subgraph_internal(&workspace_graph, &visited, config).await?;
        if config.reverse_order() {
            let mut rev = layers.clone();
            rev.reverse();
            Ok(rev)
        } else {
            Ok(layers)
        }
    }
}
