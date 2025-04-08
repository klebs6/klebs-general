// ---------------- [ File: workspacer-topo/src/layered_subgraph_internal.rs ]
crate::ix!();

pub async fn layered_subgraph_internal(
    graph:   &mut Graph<String, ()>,
    visited: &HashSet<NodeIndex>,
    config:  &TopologicalSortConfig
) -> Result<Vec<Vec<String>>, WorkspaceError> {
    trace!("entering layered_subgraph_internal");
    let mut layers = Vec::new();
    let mut in_degs = vec![0; graph.node_count()];
    let mut count_subgraph_nodes = 0usize;

    // initialize in_degs only for those nodes we want to consider:
    //   - always include visited nodes
    //   - include unvisited nodes *only* if remove_unwanted_from_graph=false
    //     so that we keep them in the graph but skip them from final output
    for idx in graph.node_indices() {
        if visited.contains(&idx) {
            // node is visited => definitely included
            count_subgraph_nodes += 1;
            let inc_count = graph
                .neighbors_directed(idx, Incoming)
                .filter(|p| visited.contains(p) || !visited.contains(p) && !*config.remove_unwanted_from_graph())
                .count();
            in_degs[idx.index()] = inc_count;
        } else if !*config.remove_unwanted_from_graph() {
            // unvisited, but remove_unwanted_from_graph=false => keep in graph for layering,
            // though we skip it in final output if there's a filter or if it's not visited
            count_subgraph_nodes += 1;
            let inc_count = graph
                .neighbors_directed(idx, Incoming)
                .filter(|p| visited.contains(p) || (!visited.contains(p) && !*config.remove_unwanted_from_graph()))
                .count();
            in_degs[idx.index()] = inc_count;
        } else {
            // unvisited AND remove_unwanted_from_graph=true => skip from layering
            in_degs[idx.index()] = usize::MAX;
        }
    }

    debug!("layered_subgraph_internal => count_subgraph_nodes={}", count_subgraph_nodes);

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
            return Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                cycle_node_id: NodeIndex::new(0),
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
            // filter checks
            if let Some(filter) = &config.filter_fn() {
                // If remove_unwanted_from_graph=true, we presumably removed unvisited nodes above.
                // But if there's also a filter, skip if it fails.
                if *config.remove_unwanted_from_graph() && !(filter)(crate_name) {
                    debug!("Skipping node={} because filter failed + remove_unwanted_from_graph", crate_name);
                    continue;
                }
                // If remove_unwanted_from_graph=false, skip from final if not visited or fails filter
                if !*config.remove_unwanted_from_graph() && !visited.contains(&n) {
                    debug!("Skipping node={} because it's unvisited + remove_unwanted_from_graph=false", crate_name);
                    continue;
                }
                if !*config.remove_unwanted_from_graph() && !(filter)(crate_name) {
                    debug!("Skipping node={} because filter failed + remove_unwanted_from_graph=false", crate_name);
                    continue;
                }
            } else {
                // no filter => skip from final if remove_unwanted_from_graph=false and not visited
                if !*config.remove_unwanted_from_graph() && !visited.contains(&n) {
                    debug!("Skipping node={} because it's unvisited + no filter + remove_unwanted_from_graph=false", crate_name);
                    continue;
                }
            }
            layer_strings.push(crate_name.clone());
        }
        if !layer_strings.is_empty() {
            layers.push(layer_strings);
        }

        // remove them from layering
        for &n in &layer_nodes {
            let degval = &mut in_degs[n.index()];
            if *degval != usize::MAX {
                *degval = usize::MAX;
                remaining -= 1;
                let node_str = &mut graph[n];
                *node_str = "".to_string();
                for neigh in graph.neighbors_directed(n, Outgoing) {
                    if in_degs[neigh.index()] != usize::MAX {
                        in_degs[neigh.index()] = in_degs[neigh.index()].saturating_sub(1);
                    }
                }
            }
        }
    }

    Ok(layers)
}

#[cfg(test)]
mod test_layered_subgraph_internal_exhaustive {
    use super::*;

    /// Helper to quickly add an edge from `src` to `dst` in the graph.
    /// Each node is `String`. If not present, we add it. Returns (src_idx, dst_idx).
    fn add_edge_string(
        graph: &mut Graph<String, ()>,
        name_to_idx: &mut HashMap<String, NodeIndex>,
        src_name: &str,
        dst_name: &str,
    ) {
        let src_idx = *name_to_idx
            .entry(src_name.to_string())
            .or_insert_with(|| graph.add_node(src_name.to_string()));
        let dst_idx = *name_to_idx
            .entry(dst_name.to_string())
            .or_insert_with(|| graph.add_node(dst_name.to_string()));
        // Avoid duplicates
        if !graph.edges_connecting(src_idx, dst_idx).next().is_some() {
            graph.add_edge(src_idx, dst_idx, ());
        }
    }

    /// Minimal helper to run `layered_subgraph_internal` and return the layers (Vec<Vec<String>>).
    async fn run_layered_subgraph_internal_test(
        graph: &mut Graph<String, ()>,
        visited: &HashSet<NodeIndex>,
        config: &TopologicalSortConfig,
    ) -> Result<Vec<Vec<String>>, WorkspaceError> {
        trace!("run_layered_subgraph_internal_test - start");
        layered_subgraph_internal(graph, visited, config).await
    }

    // ---------------------------------------------------------------------------
    // Test 1) Empty graph => no layers
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_empty_graph() {
        trace!("test_empty_graph - start");
        let mut graph = Graph::<String, ()>::new();
        let visited = HashSet::new(); // nothing visited
        let config = TopologicalSortConfigBuilder::default()
            .build()
            .unwrap();

        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("Empty graph => no cycle, just empty layers");
        assert_eq!(layers.len(), 0, "No nodes => no layers");
        info!("test_empty_graph => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 2) Single node, visited => we get exactly one layer containing that node
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_single_node_visited() {
        trace!("test_single_node_visited - start");
        let mut graph = Graph::<String, ()>::new();
        let idx = graph.add_node("solo".to_string());
        let mut visited = HashSet::new();
        visited.insert(idx);

        let config = TopologicalSortConfigBuilder::default()
            .build()
            .unwrap();

        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("Single visited node => should yield 1 layer");
        assert_eq!(layers.len(), 1, "Expected exactly 1 layer");
        assert_eq!(layers[0], vec!["solo"]);
        info!("test_single_node_visited => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 3) Single node, unvisited, remove_unwanted_from_graph=false => yields 0 layers
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_single_node_unvisited_skip() {
        trace!("test_single_node_unvisited_skip - start");
        let mut graph = Graph::<String, ()>::new();
        graph.add_node("solo".to_string());
        let visited = HashSet::new(); // node not visited

        let config = TopologicalSortConfigBuilder::default()
            .remove_unwanted_from_graph(false)
            .build()
            .unwrap();

        // We skip unvisited if remove_unwanted_from_graph=false
        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        assert_eq!(layers.len(), 0, "Node is unvisited => skip => 0 layers");
        info!("test_single_node_unvisited_skip => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 4) Single node, unvisited, remove_unwanted_from_graph=true => node is removed from graph => 0 layers
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_single_node_unvisited_removed() {
        trace!("test_single_node_unvisited_removed - start");
        let mut graph = Graph::<String, ()>::new();
        graph.add_node("solo".to_string());
        let visited = HashSet::new(); // node not visited

        let config = TopologicalSortConfigBuilder::default()
            .remove_unwanted_from_graph(true)
            .build()
            .unwrap();

        // Because remove_unwanted_from_graph=true, that node is effectively removed before layering
        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        assert_eq!(layers.len(), 0, "Unvisited node removed => 0 layers");
        info!("test_single_node_unvisited_removed => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 5) Three nodes in a chain, all visited => expect layering [n1], [n2], [n3]
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_chain_all_visited() {
        trace!("test_chain_all_visited - start");
        let mut graph = Graph::<String, ()>::new();
        let mut map = HashMap::new();
        // We'll do chain: A -> B -> C (A has an edge to B, B has an edge to C).
        // That means in layering, layer1=[A], layer2=[B], layer3=[C].
        add_edge_string(&mut graph, &mut map, "A", "B");
        add_edge_string(&mut graph, &mut map, "B", "C");

        let visited: HashSet<NodeIndex> = graph.node_indices().collect(); // everything visited

        let config = TopologicalSortConfigBuilder::default()
            .build()
            .unwrap();

        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        assert_eq!(layers.len(), 3, "Should have 3 layers in a simple chain");
        // The alphabetical ordering in each layer only has one node, so trivial.
        assert_eq!(layers[0], vec!["A"]);
        assert_eq!(layers[1], vec!["B"]);
        assert_eq!(layers[2], vec!["C"]);
        info!("test_chain_all_visited => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 6) A simple cycle => layering fails with cycle error
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_simple_cycle() {
        trace!("test_simple_cycle - start");
        let mut graph = Graph::<String, ()>::new();
        let mut map = HashMap::new();
        add_edge_string(&mut graph, &mut map, "A", "B");
        add_edge_string(&mut graph, &mut map, "B", "C");
        add_edge_string(&mut graph, &mut map, "C", "A"); // cycle A->B->C->A

        let visited: HashSet<NodeIndex> = graph.node_indices().collect(); // everything visited
        let config = TopologicalSortConfigBuilder::default()
            .build()
            .unwrap();

        let result = run_layered_subgraph_internal_test(&mut graph, &visited, &config).await;
        assert!(result.is_err(), "Cycle => should fail");
        match result {
            Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph{..}) => {
                // expected
            }
            other => panic!("Expected cycle error, got: {other:?}"),
        }
        info!("test_simple_cycle => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 7) Partial visited with remove_unwanted_from_graph=false => unvisited is skipped
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_partial_visited_skip_unwanted() {
        trace!("test_partial_visited_skip_unwanted - start");
        let mut graph = Graph::<String, ()>::new();
        let mut map = HashMap::new();
        add_edge_string(&mut graph, &mut map, "A", "B"); // chain A->B
        add_edge_string(&mut graph, &mut map, "X", "Y"); // separate chain X->Y
        // We'll only mark A,B visited, but not X,Y
        let mut visited = HashSet::new();
        visited.insert(map["A"]);
        visited.insert(map["B"]);

        let config = TopologicalSortConfigBuilder::default()
            .remove_unwanted_from_graph(false)
            .build()
            .unwrap();

        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        // We'll see layering only for the visited subgraph => [A], [B]
        // X, Y are unvisited => skip them in final output
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0], vec!["A"]);
        assert_eq!(layers[1], vec!["B"]);
        info!("test_partial_visited_skip_unwanted => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 8) Partial visited with remove_unwanted_from_graph=true => unvisited is removed
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_partial_visited_remove_unwanted() {
        trace!("test_partial_visited_remove_unwanted - start");
        let mut graph = Graph::<String, ()>::new();
        let mut map = HashMap::new();
        add_edge_string(&mut graph, &mut map, "A", "B");
        add_edge_string(&mut graph, &mut map, "X", "Y");
        // We'll only mark A,B visited, not X,Y
        let mut visited = HashSet::new();
        visited.insert(map["A"]);
        visited.insert(map["B"]);

        let config = TopologicalSortConfigBuilder::default()
            .remove_unwanted_from_graph(true)
            .build()
            .unwrap();

        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        // We'll see layering for [A], [B]. The X->Y part is physically removed from the graph
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0], vec!["A"]);
        assert_eq!(layers[1], vec!["B"]);
        info!("test_partial_visited_remove_unwanted => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 9) Check filter_fn that excludes "skipMe"
    //         - remove_unwanted_from_graph=false => skip in final layer, but keep edges
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_filter_fn_skip_in_output_only() {
        trace!("test_filter_fn_skip_in_output_only - start");
        let mut graph = Graph::<String, ()>::new();
        let mut map = HashMap::new();
        // Edges: skipMe -> realA -> realB
        add_edge_string(&mut graph, &mut map, "skipMe", "realA");
        add_edge_string(&mut graph, &mut map, "realA", "realB");

        let visited: HashSet<NodeIndex> = graph.node_indices().collect();
        let filter_closure: Arc<dyn Fn(&str) -> bool + Send + Sync> =
            Arc::new(|name: &str| name != "skipMe");
        let config = TopologicalSortConfigBuilder::default()
            .filter_fn(Some(filter_closure))
            .remove_unwanted_from_graph(false)
            .build()
            .unwrap();

        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        // We'll see layering: layer1 has skipMe in-degree=0, but skipMe fails filter => skip in output
        // Then realA is next => output, then realB => output
        // So the final layered structure => [realA], [realB]
        assert_eq!(layers.len(), 2, "We skip skipMe from final output");
        assert_eq!(layers[0], vec!["realA"]);
        assert_eq!(layers[1], vec!["realB"]);
        info!("test_filter_fn_skip_in_output_only => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 10) Check filter_fn that excludes "skipMe"
    //          - remove_unwanted_from_graph=true => skipMe is removed from graph
    //            so realA, realB might get an earlier in-degree=0
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_filter_fn_remove_unwanted() {
        trace!("test_filter_fn_remove_unwanted - start");
        let mut graph = Graph::<String, ()>::new();
        let mut map = HashMap::new();
        // Edges: skipMe -> realA -> realB
        add_edge_string(&mut graph, &mut map, "skipMe", "realA");
        add_edge_string(&mut graph, &mut map, "realA", "realB");

        let visited: HashSet<NodeIndex> = graph.node_indices().collect();
        let filter_closure: Arc<dyn Fn(&str) -> bool + Send + Sync> =
            Arc::new(|name: &str| name != "skipMe");
        let config = TopologicalSortConfigBuilder::default()
            .filter_fn(Some(filter_closure))
            .remove_unwanted_from_graph(true)
            .build()
            .unwrap();

        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        // skipMe is removed from the graph => realA might have in-degree=0 immediately
        // layering => [realA], [realB]
        assert_eq!(layers.len(), 2, "We removed skipMe => realA is first layer");
        assert_eq!(layers[0], vec!["realA"]);
        assert_eq!(layers[1], vec!["realB"]);
        info!("test_filter_fn_remove_unwanted => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 11) weighting_fn example
    //          We have multiple in-degree=0 nodes => the weighting decides the order
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_weighting_fn_order_in_layer() {
        trace!("test_weighting_fn_order_in_layer - start");
        let mut graph = Graph::<String, ()>::new();
        let mut map = HashMap::new();
        // We'll add edges: A->X, B->X (X depends on A & B).
        // So A,B are in-degree=0 initially. They will appear in the same layer.
        // The alphabetical approach would yield [A, B] by name. Let's override with weighting.

        add_edge_string(&mut graph, &mut map, "A", "X");
        add_edge_string(&mut graph, &mut map, "B", "X");
        let visited: HashSet<NodeIndex> = graph.node_indices().collect();

        // weighting_fn => say "B" => weight 10, "A" => weight 20 => smaller => earlier
        // so if weighting is ascending order by weight => "B" < "A"
        let closure: Arc<dyn Fn(&str) -> u32 + Send + Sync> = Arc::new(|name: &str| {
            match name {
                "A" => 20,
                "B" => 10,
                _ => 0,
            }
        });

        let config = TopologicalSortConfigBuilder::default()
            .weighting_fn(Some(closure))
            .build()
            .unwrap();

        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        // layer1 => [B, A] because B has weight=10, A has weight=20 => B first
        // layer2 => [X]
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0], vec!["B","A"]);
        assert_eq!(layers[1], vec!["X"]);
        info!("test_weighting_fn_order_in_layer => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 12) finalize a multi-layer scenario with partial visited + filter + weighting
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_multi_factor_combined() {
        trace!("test_multi_factor_combined - start");
        // We'll build a graph with nodes: A, B, C, D, E, F
        // Edges:
        //   A->C, B->C => C depends on A,B
        //   C->D => D depends on C
        //   E->F => F depends on E
        let mut graph = Graph::<String, ()>::new();
        let mut map = HashMap::new();
        add_edge_string(&mut graph, &mut map, "A", "C");
        add_edge_string(&mut graph, &mut map, "B", "C");
        add_edge_string(&mut graph, &mut map, "C", "D");
        add_edge_string(&mut graph, &mut map, "E", "F");

        // We'll only "visit" A,B,C,D but not E,F => so E,F are unvisited
        let mut visited = HashSet::new();
        visited.insert(map["A"]);
        visited.insert(map["B"]);
        visited.insert(map["C"]);
        visited.insert(map["D"]);

        // We'll filter out "B" => skip it or remove it
        let filter_fn: Arc<dyn Fn(&str)->bool + Send + Sync> =
            Arc::new(|name: &str| name != "B");

        // We'll also define weighting: A => 5, B => 1, C => 2, D => 10, E => 99, F => 99
        // so that among any in-degree=0 layer, we pick ascending by that numeric weight
        let weighting: Arc<dyn Fn(&str)->u32 + Send + Sync> = Arc::new(|name: &str| {
            match name {
                "A" => 5,
                "B" => 1,
                "C" => 2,
                "D" => 10,
                "E" => 99,
                "F" => 99,
                _ => 0,
            }
        });

        let config = TopologicalSortConfigBuilder::default()
            .filter_fn(Some(filter_fn))
            .weighting_fn(Some(weighting))
            .remove_unwanted_from_graph(false) // we skip unvisited
            .build()
            .unwrap();

        // Expect layering ignoring E,F entirely => ignoring B from final output, but B is present in the graph
        // We'll see something like: first layer => [A], second => [C], third => [D].
        // B is present but fails the filter => not in final output. Also, E,F are unvisited => skip.
        let layers = run_layered_subgraph_internal_test(&mut graph, &visited, &config)
            .await
            .expect("No cycle => success");
        // Let’s see if B influences the in-degree of C. In-degree(C)=2 => A,B. But B is there in the graph,
        // so we do not remove it from the graph, but we do skip it from final output. The layering sees:
        //   layer0 => [A,B], then we only add "A" to final if it passes filter => "B" is omitted from final output
        //   after removing them => "C" => layer1 => [C], then => layer2 => [D].
        // So final => 3 layers, but each layer might omit B from the final strings.
        // We expect [ ["A"], ["C"], ["D"] ]
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec!["A"]);
        assert_eq!(layers[1], vec!["C"]);
        assert_eq!(layers[2], vec!["D"]);
        info!("test_multi_factor_combined => passed");
    }
}
