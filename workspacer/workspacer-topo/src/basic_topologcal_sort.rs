crate::ix!();

// ----------------------------------------------------------------------------------
// Implement BasicTopologicalSort for Workspace<P, H>
// ----------------------------------------------------------------------------------
#[async_trait]
impl<P,H> BasicTopologicalSort for Workspace<P,H>
where
    P: AsRef<Path> + From<std::path::PathBuf> + Send + Sync,
    H: Send + Sync + 'static,
    Workspace<P,H>: GenerateDependencyTree<Tree=petgraph::Graph<String,()>,Error=WorkspaceError>,
{
    async fn topological_order_crate_names(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<String>, WorkspaceError> {
        trace!("Workspace::topological_order_crate_names - start");
        if config.layering_enabled() {
            // flatten the layered version
            let layered = self.layered_topological_order_crate_names(config).await?;
            let mut flat = Vec::new();
            for layer in layered {
                for crate_name in layer {
                    flat.push(crate_name);
                }
            }
            if config.reverse_order() {
                flat.reverse();
            }
            return Ok(flat);
        }

        // Non-layered approach => standard petgraph toposort
        let mut graph = self.generate_dependency_tree().await?;
        debug!("Workspace graph node_count={}", graph.node_count());

        // apply filter removal
        if let Some(filter) = &config.filter_fn() {
            if config.remove_unwanted_from_graph() {
                let mut to_remove = Vec::new();
                for idx in graph.node_indices() {
                    let crate_name = &graph[idx];
                    if !(filter)(crate_name) {
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

        // run toposort
        let sorted_indices = match toposort(&graph, None) {
            Ok(list) => list,
            Err(Cycle{ node_id }) => {
                error!("Cycle detected at node={:?}", node_id);
                return Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                    cycle_node_id: node_id,
                });
            }
        };

        // build final list
        let mut result = Vec::new();
        for idx in sorted_indices {
            let name = &graph[idx];
            if let Some(filter) = &config.filter_fn() {
                if !config.remove_unwanted_from_graph() && !(filter)(name) {
                    continue;
                }
            }
            result.push(name.clone());
        }

        if config.reverse_order() {
            result.reverse();
        }

        info!("Workspace::topological_order_crate_names => {} crates", result.len());
        Ok(result)
    }
}
