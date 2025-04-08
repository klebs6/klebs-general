crate::ix!();

// ----------------------------------------------------------------------------------
// FocusCrateTopologicalSort for Workspace<P,H> => subgraph of ancestors leading to focus crate
// ----------------------------------------------------------------------------------
#[async_trait]
impl<P,H> FocusCrateTopologicalSort for Workspace<P,H>
where
    P: AsRef<Path> + From<std::path::PathBuf> + Send + Sync,
    H: Send + Sync + 'static,
    Workspace<P,H>: GenerateDependencyTree<Tree=petgraph::Graph<String,()>,Error=WorkspaceError>,
{
    async fn topological_order_upto_crate(
        &self,
        config: &TopologicalSortConfig,
        focus_crate_name: &str
    ) -> Result<Vec<String>, WorkspaceError> {
        trace!("Workspace::topological_order_upto_crate - focus={}", focus_crate_name);
        let mut graph = self.generate_dependency_tree().await?;
        // find node
        let mut focus_idx = None;
        for idx in graph.node_indices() {
            if graph[idx] == focus_crate_name {
                focus_idx = Some(idx);
                break;
            }
        }
        if focus_idx.is_none() {
            warn!("Focus crate '{}' not found => returning empty result", focus_crate_name);
            return Ok(vec![]);
        }

        let focus_idx = focus_idx.unwrap();
        // gather ancestors
        let mut visited = HashSet::new();
        visited.insert(focus_idx);
        let mut stack = vec![focus_idx];
        while let Some(cur) = stack.pop() {
            for pred in graph.neighbors_directed(cur, Direction::Incoming) {
                if !visited.contains(&pred) {
                    visited.insert(pred);
                    stack.push(pred);
                }
            }
        }

        // remove unvisited if remove_unwanted_from_graph
        if config.remove_unwanted_from_graph() {
            let mut to_remove = Vec::new();
            for idx in graph.node_indices() {
                if !visited.contains(&idx) {
                    to_remove.push(idx);
                }
            }
            to_remove.sort_by_key(|i| i.index());
            to_remove.reverse();
            for idx in to_remove {
                graph.remove_node(idx);
            }
        }

        if config.layering_enabled() {
            // we'll do layering subgraph, then flatten
            let layers = layered_subgraph_internal(&graph, &visited, config).await?;
            let mut flat = Vec::new();
            for layer in layers {
                for crate_name in layer {
                    flat.push(crate_name);
                }
            }
            return Ok(flat);
        } else {
            // normal toposort
            let sorted_idx = match toposort(&graph, None) {
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
                    let name = &graph[idx];
                    if config.remove_unwanted_from_graph() && !(filter)(name) {
                        continue;
                    }
                    if !config.remove_unwanted_from_graph() && !(filter)(name) {
                        continue;
                    }
                }
                result.push(graph[idx].clone());
            }
            if config.reverse_order() {
                result.reverse();
            }
            return Ok(result);
        }
    }

    async fn layered_topological_order_upto_crate(
        &self,
        config: &TopologicalSortConfig,
        focus_crate_name: &str
    ) -> Result<Vec<Vec<String>>, WorkspaceError> {
        trace!("Workspace::layered_topological_order_upto_crate - focus={}", focus_crate_name);
        let mut graph = self.generate_dependency_tree().await?;
        // find
        let mut focus_idx = None;
        for idx in graph.node_indices() {
            if graph[idx] == focus_crate_name {
                focus_idx = Some(idx);
                break;
            }
        }
        if focus_idx.is_none() {
            warn!("Focus crate '{}' not found => returning empty layering", focus_crate_name);
            return Ok(vec![]);
        }
        let focus_idx = focus_idx.unwrap();

        // gather ancestors
        let mut visited = HashSet::new();
        visited.insert(focus_idx);
        let mut stack = vec![focus_idx];
        while let Some(cur) = stack.pop() {
            for pred in graph.neighbors_directed(cur, Direction::Incoming) {
                if !visited.contains(&pred) {
                    visited.insert(pred);
                    stack.push(pred);
                }
            }
        }

        // remove unvisited if remove
        if config.remove_unwanted_from_graph() {
            let mut to_remove = Vec::new();
            for idx in graph.node_indices() {
                if !visited.contains(&idx) {
                    to_remove.push(idx);
                }
            }
            to_remove.sort_by_key(|i| i.index());
            to_remove.reverse();
            for idx in to_remove {
                graph.remove_node(idx);
            }
        }

        let layers = layered_subgraph_internal(&graph, &visited, config).await?;
        if config.reverse_order() {
            let mut rev = layers.clone();
            rev.reverse();
            return Ok(rev);
        }
        Ok(layers)
    }
}
