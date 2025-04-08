// ---------------- [ File: workspacer-topo/src/topological_sort_internal_deps.rs ]
crate::ix!();

// ======================================
// 7) TopologicalSortInternalDeps for CrateHandle
// using BFS-based add_crate_and_deps to build a subgraph. 
// Then topological or layered sort that subgraph.
// ======================================
#[async_trait]
impl TopologicalSortInternalDeps for CrateHandle {
    async fn topological_sort_internal_deps(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<String>, WorkspaceError> {
        trace!("CrateHandle::topological_sort_internal_deps => '{}'", self.name());

        let mut graph = Graph::<String, ()>::new();
        let crate_idx = add_crate_and_deps(&mut graph, self).await?;
        debug!("Root crate index={:?} in subgraph", crate_idx);

        // We can do a standard toposort, with optional layering approach, etc.
        if *config.layering_enabled() {
            // layering => flatten
            let visited: HashSet<NodeIndex> = graph.node_indices().collect();
            let mut layers = layered_subgraph_internal(&mut graph, &visited, config).await?;
            let mut flat = Vec::new();
            for layer in layers.drain(..) {
                flat.extend(layer);
            }
            Ok(flat)
        } else {
            // normal toposort
            match toposort(&graph, None) {
                Ok(list) => {
                    let mut result = Vec::new();
                    for idx in list {
                        let crate_name = &graph[idx];
                        // skip if filter fails
                        if let Some(filter) = &config.filter_fn() {
                            if *config.remove_unwanted_from_graph() && !(filter)(crate_name) {
                                continue;
                            }
                            if !config.remove_unwanted_from_graph() && !(filter)(crate_name) {
                                continue;
                            }
                        }
                        result.push(crate_name.clone());
                    }
                    if *config.reverse_order() {
                        result.reverse();
                    }
                    Ok(result)
                }
                Err(ref cycle) => {
                    let node_id = cycle.node_id();
                    error!("Cycle => node={:?}", node_id);
                    Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                        cycle_node_id: node_id,
                    })
                }
            }
        }
    }

    async fn layered_topological_order_upto_self(
        &self,
        config: &TopologicalSortConfig
    ) -> Result<Vec<Vec<String>>, WorkspaceError> {
        trace!("CrateHandle::layered_topological_order_upto_self => '{}'", self.name());
        let mut graph = Graph::<String, ()>::new();
        let crate_idx = add_crate_and_deps(&mut graph, self).await?;
        debug!("Root crate index={:?} in layered approach", crate_idx);

        let visited: HashSet<NodeIndex> = graph.node_indices().collect();
        if *config.remove_unwanted_from_graph() {
            let mut to_remove: Vec<_> = graph
                .node_indices()
                .filter(|&idx| !visited.contains(&idx))
                .collect();
            to_remove.sort_by_key(|i| i.index());
            to_remove.reverse();
            for idx in to_remove {
                graph.remove_node(idx);
            }
        }

        let mut layers = layered_subgraph_internal(&mut graph, &visited, config).await?;
        if *config.reverse_order() {
            layers.reverse();
        }
        Ok(layers)
    }
}

#[cfg(test)]
mod test_topological_sort_internal_deps_exhaustive {
    use super::*;
    use std::collections::HashSet;
    use tracing::*;

    /// Helper: create a single "root" crate with some direct local deps, parse the
    /// resulting CrateHandle, then call `topological_sort_internal_deps` on it with
    /// the given `config`.
    async fn run_topo_internal_deps(
        root_name: &str,
        direct_deps: &[&str],
        broken_toml: bool,
        config: &TopologicalSortConfig,
    ) -> Result<Vec<String>, WorkspaceError> {
        trace!("run_topo_internal_deps(root='{root_name}', direct_deps={direct_deps:?}, broken_toml={broken_toml})");

        // 1) Create the workspace & single root crate
        let crate_handle = {
            let handle = create_workspace_and_get_handle(root_name, direct_deps, broken_toml).await?;
            handle
        };

        // 2) Now we call `topological_sort_internal_deps` on that single crate handle
        crate_handle.topological_sort_internal_deps(config).await
    }

    /// Similar helper for layered approach: calls `layered_topological_order_upto_self`
    async fn run_layered_upto_self(
        root_name: &str,
        direct_deps: &[&str],
        broken_toml: bool,
        config: &TopologicalSortConfig,
    ) -> Result<Vec<Vec<String>>, WorkspaceError> {
        trace!("run_layered_upto_self(root='{root_name}', direct_deps={direct_deps:?}, broken_toml={broken_toml})");

        let crate_handle = {
            let handle = create_workspace_and_get_handle(root_name, direct_deps, broken_toml).await?;
            handle
        };

        crate_handle.layered_topological_order_upto_self(config).await
    }

    // ---------------------------------------------------------------------------
    // Test 1) Single crate, no deps => topological_sort_internal_deps => just [root]
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_single_crate_no_deps() {
        trace!("test_single_crate_no_deps - start");
        let config = TopologicalSortConfigBuilder::default().build().unwrap();
        let result = run_topo_internal_deps("root_crate", &[], false, &config)
            .await
            .expect("Single crate => no deps => BFS => [root_crate] only");
        assert_eq!(result, vec!["root_crate"]);
        info!("test_single_crate_no_deps => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 2) Single crate, multiple direct deps => BFS => edges => dep->root => topological => [dep1,dep2,root]
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_multiple_direct_deps() {
        trace!("test_multiple_direct_deps - start");
        let config = TopologicalSortConfigBuilder::default().build().unwrap();
        let deps = vec!["depA", "depB"];
        let result = run_topo_internal_deps("root_crate", &deps, false, &config)
            .await
            .expect("BFS => multiple direct deps => no cycle");
        let mut sorted = result.clone();
        sorted.sort();
        let mut expected = vec!["depA","depB","root_crate"];
        expected.sort();
        assert_eq!(sorted, expected, "Same set of nodes");
        info!("test_multiple_direct_deps => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 3) Single crate, repeated direct deps => skip duplicates
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_repeated_deps() {
        trace!("test_repeated_deps - start");
        let config = TopologicalSortConfigBuilder::default().build().unwrap();
        let deps = vec!["depA","depA","depB"];
        let result = run_topo_internal_deps("root_crate", &deps, false, &config)
            .await
            .expect("Should skip duplicates gracefully");
        let mut sorted = result.clone();
        sorted.sort();
        let mut expected = vec!["depA","depB","root_crate"];
        expected.sort();
        assert_eq!(sorted, expected);
        info!("test_repeated_deps => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 4) broken Cargo.toml => parse error => BFS fails
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_broken_cargo_toml() {
        trace!("test_broken_cargo_toml - start");
        let config = TopologicalSortConfigBuilder::default().build().unwrap();
        // We force a parse error
        let result = run_topo_internal_deps("root_crate", &["depX"], true, &config).await;
        assert!(result.is_err(), "Broken Cargo.toml => should fail");
        match result.err().unwrap() {
            WorkspaceError::CrateError(e) => {
                debug!("Got expected BFS error => CrateError: {e:?}");
            }
            other => panic!("Expected CrateError for broken toml => got {other:?}"),
        }
        info!("test_broken_cargo_toml => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 5) layering_enabled => BFS subgraph => flatten => [deps..., root]
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_layering_enabled_flat() {
        trace!("test_layering_enabled_flat - start");
        let config = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .build()
            .unwrap();
        let deps = vec!["A","B"];
        let result = run_topo_internal_deps("root_crate", &deps, false, &config)
            .await
            .expect("Layering => flatten => likely [A,B,root_crate]");
        let mut sorted = result.clone();
        sorted.sort();
        let mut expected = vec!["A","B","root_crate"];
        expected.sort();
        assert_eq!(sorted, expected);
        info!("test_layering_enabled_flat => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 6) filter_fn => skip a direct dep from final. remove_unwanted_from_graph=false => keep in BFS graph
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_filter_skip_dep() {
        trace!("test_filter_skip_dep - start");
        let filter_closure: Arc<dyn Fn(&str)->bool + Send + Sync> = Arc::new(|nm: &str| nm != "skipMe");
        let config = TopologicalSortConfigBuilder::default()
            .filter_fn(Some(filter_closure))
            .remove_unwanted_from_graph(false)
            .build()
            .unwrap();

        let deps = vec!["skipMe","keepMe"];
        let result = run_topo_internal_deps("root_crate", &deps, false, &config)
            .await
            .expect("BFS => skipMe => but filter excludes skipMe from final output");
        let mut sorted = result.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["keepMe","root_crate"]);
        info!("test_filter_skip_dep => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 7) filter_fn => remove dep from BFS entirely => remove_unwanted_from_graph=true
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_filter_remove_dep() {
        trace!("test_filter_remove_dep - start");
        let filter_closure: Arc<dyn Fn(&str)->bool + Send + Sync> = Arc::new(|nm: &str| nm != "removeMe");
        let config = TopologicalSortConfigBuilder::default()
            .filter_fn(Some(filter_closure))
            .remove_unwanted_from_graph(true)
            .build()
            .unwrap();

        let deps = vec!["removeMe","keepMe"];
        let result = run_topo_internal_deps("root_crate", &deps, false, &config)
            .await
            .expect("BFS => removeMe => removed from graph");
        let mut sorted = result.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["keepMe","root_crate"]);
        info!("test_filter_remove_dep => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 8) weighting_fn => reorder direct deps in layering? But if layering_enabled=false, no effect on toposort order
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_weighting_fn_no_layering() {
        trace!("test_weighting_fn_no_layering - start");
        let weighting: Arc<dyn Fn(&str)->u32 + Send + Sync> = Arc::new(|nm: &str| match nm {
            "lowDep" => 10,
            "highDep" => 99,
            "root_crate" => 5,
            _ => 0,
        });
        let config = TopologicalSortConfigBuilder::default()
            .weighting_fn(Some(weighting))
            .build()
            .unwrap();

        let deps = vec!["lowDep","highDep"];
        let result = run_topo_internal_deps("root_crate", &deps, false, &config)
            .await
            .expect("No layering => ignoring weighting => BFS might yield any order for [lowDep, highDep] + root");
        let mut sorted = result.clone();
        sorted.sort();
        let mut expected = vec!["highDep","lowDep","root_crate"];
        expected.sort();
        assert_eq!(sorted, expected);
        info!("test_weighting_fn_no_layering => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 9) weighting_fn + layering => reorder direct deps in each layer
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_weighting_fn_with_layering() {
        trace!("test_weighting_fn_with_layering - start");
        let weighting: Arc<dyn Fn(&str)->u32 + Send + Sync> = Arc::new(|nm: &str| {
            match nm {
                "depA" => 50,
                "depB" => 10,
                _ => 1,
            }
        });
        let config = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .weighting_fn(Some(weighting))
            .build()
            .unwrap();

        let deps = vec!["depA","depB"];
        let result = run_topo_internal_deps("root_crate", &deps, false, &config)
            .await
            .expect("layering => weighted => depB first in the first layer, then root_crate");
        // Flatten => depB, depA, root_crate
        assert_eq!(result, vec!["depB","depA","root_crate"]);
        info!("test_weighting_fn_with_layering => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 10) reverse_order => typical BFS => normal => [depA, depB, root], reversed => [root, depB, depA]
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_reverse_order() {
        trace!("test_reverse_order - start");
        let deps = vec!["depA","depB"];
        let base_cfg = TopologicalSortConfigBuilder::default().build().unwrap();
        let normal = run_topo_internal_deps("root_crate", &deps, false, &base_cfg)
            .await
            .expect("normal BFS => [depA, depB, root_crate] or variation");

        let rev_cfg = TopologicalSortConfigBuilder::default()
            .reverse_order(true)
            .build()
            .unwrap();
        let reversed = run_topo_internal_deps("root_crate", &deps, false, &rev_cfg)
            .await
            .expect("reversed BFS => [root_crate, depB, depA] or variation");

        // We'll compare sets, not exact sequence
        let mut normal_sorted = normal.clone();
        normal_sorted.sort();
        let mut rev_sorted = reversed.clone();
        rev_sorted.sort();
        assert_eq!(normal_sorted, rev_sorted, "same sets of nodes, reversed order");
        info!("test_reverse_order => normal={normal:?}, reversed={reversed:?}");
        info!("test_reverse_order => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 11) layered_topological_order_upto_self => single-level BFS => layering with direct deps
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_layered_topological_order_upto_self_simple() {
        trace!("test_layered_topological_order_upto_self_simple - start");
        let deps = vec!["A","B"];
        let config = TopologicalSortConfigBuilder::default()
            .build()
            .unwrap();
        let layers = run_layered_upto_self("root_crate", &deps, false, &config)
            .await
            .expect("No cycle => layering => [A,B],[root_crate]");
        assert_eq!(layers.len(), 2);
        let mut layer0 = layers[0].clone();
        layer0.sort();
        assert_eq!(layer0, vec!["A","B"]);
        assert_eq!(layers[1], vec!["root_crate"]);
        info!("test_layered_topological_order_upto_self_simple => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 12) layered_topological_order_upto_self => weighting + filter + reverse
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_layered_topological_order_upto_self_complex() {
        trace!("test_layered_topological_order_upto_self_complex - start");

        let filter_fn: Arc<dyn Fn(&str)->bool + Send + Sync> = Arc::new(|nm: &str| nm != "skipMe");
        let weighting: Arc<dyn Fn(&str)->u32 + Send + Sync> = Arc::new(|nm: &str| match nm {
            "one" => 2,
            "two" => 1,
            _ => 99,
        });
        let config = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .reverse_order(true)
            .filter_fn(Some(filter_fn))
            .weighting_fn(Some(weighting))
            .remove_unwanted_from_graph(false)
            .build()
            .unwrap();

        let deps = vec!["one","two","skipMe"];
        // BFS => edges one->root, two->root, skipMe->root => layering => [two, one, skipMe],[root] ignoring skipMe in final output => effectively [two, one],[root], then reversed => [root],[two, one]
        let layers = run_layered_upto_self("root_crate", &deps, false, &config)
            .await
            .expect("BFS => layering => reversed => skip skipMe");
        // final => [root_crate],[two,one]
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0], vec!["root_crate"]);
        let mut second = layers[1].clone();
        second.sort();
        assert_eq!(second, vec!["one","two"]);
        info!("test_layered_topological_order_upto_self_complex => passed");
    }
}
