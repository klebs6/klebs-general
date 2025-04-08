crate::ix!();

// ----------------------------------------------------------------------------------
// Implement LayeredTopologicalSort for Workspace<P,H>
// ----------------------------------------------------------------------------------
#[async_trait]
impl<P,H> LayeredTopologicalSort for Workspace<P,H>
where
    P: AsRef<Path> + From<std::path::PathBuf> + Send + Sync,
    H: Send + Sync + 'static,
    Workspace<P,H>: GenerateDependencyTree<Tree=petgraph::Graph<String,()>,Error=WorkspaceError>,
{
    async fn layered_topological_order_crate_names(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<Vec<String>>, WorkspaceError> {
        trace!("Workspace::layered_topological_order_crate_names - start");
        let mut graph = self.generate_dependency_tree().await?;
        debug!("Graph node_count={}", graph.node_count());

        // remove filter if needed
        if let Some(filter) = &config.filter_fn() {
            if config.remove_unwanted_from_graph() {
                let mut to_remove = Vec::new();
                for idx in graph.node_indices() {
                    if !(filter)(&graph[idx]) {
                        to_remove.push(idx);
                    }
                }
                to_remove.sort_by_key(|i| i.index());
                to_remove.reverse();
                for idx in to_remove {
                    graph.remove_node(idx);
                }
            }
        }

        // standard Kahn layering
        let mut layers = Vec::new();
        let mut in_degs = vec![0; graph.node_count()];
        for idx in graph.node_indices() {
            in_degs[idx.index()] = graph.neighbors_directed(idx, Direction::Incoming).count();
        }
        let mut unvisited_count = graph.node_count();

        while unvisited_count > 0 {
            let mut layer_nodes = Vec::new();
            for idx in graph.node_indices() {
                if in_degs[idx.index()] == 0 && !graph[idx].is_empty() {
                    layer_nodes.push(idx);
                }
            }
            if layer_nodes.is_empty() {
                error!("Layering found no zero-in-degree => cycle");
                return Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                    cycle_node_id: 0,
                });
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
                    if !config.remove_unwanted_from_graph() && !(filter)(crate_name) {
                        continue;
                    }
                }
                layer_strings.push(crate_name.clone());
            }
            if !layer_strings.is_empty() {
                layers.push(layer_strings);
            }

            // remove from layering
            for &n in &layer_nodes {
                unvisited_count -= 1;
                let node_str = graph.node_weight_mut(n).unwrap();
                *node_str = "".to_string();
                for neigh in graph.neighbors_directed(n, Direction::Outgoing) {
                    in_degs[neigh.index()] = in_degs[neigh.index()].saturating_sub(1);
                }
            }
        }

        if config.reverse_order() {
            layers.reverse();
        }
        info!("Workspace::layered_topological_order_crate_names => {} layers", layers.len());
        Ok(layers)
    }
}


