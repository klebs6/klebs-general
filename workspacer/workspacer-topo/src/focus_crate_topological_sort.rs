// ---------------- [ File: workspacer-topo/src/focus_crate_topological_sort.rs ]
crate::ix!();

#[async_trait]
impl<P, H> FocusCrateTopologicalSort for Workspace<P, H>
where
    for<'a> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'a,
    for<'a> H: CrateHandleInterface<P> + Send + Sync + 'static,
    Self: GenerateDependencyTree<Tree = Graph<String, ()>, Error = WorkspaceError>,
{
    async fn topological_order_upto_crate(
        &self,
        config: &TopologicalSortConfig,
        focus_crate_name: &str
    ) -> Result<Vec<String>, WorkspaceError> {
        trace!("Workspace::topological_order_upto_crate - focus={focus_crate_name}");

        // 1) Generate cargo graph, handle cargo cycle/empty
        let mut graph = match self.generate_dependency_tree().await {
            Ok(g) => g,
            Err(e) => {
                if let WorkspaceError::CargoMetadataError(CargoMetadataError::MetadataError { error: ref meta_err }) = e {
                    let stderr = format!("{:?}", meta_err);
                    if stderr.contains("cyclic package dependency") {
                        error!("Cargo says cycle => mapping to CycleDetectedInWorkspaceDependencyGraph");
                        return Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                            cycle_node_id: NodeIndex::new(0),
                        });
                    }
                    if stderr.contains("contains no package: The manifest is virtual, and the workspace has no members") {
                        warn!("Empty workspace => returning []");
                        return Ok(vec![]);
                    }
                }
                return Err(e);
            }
        };

        // 2) If remove_unwanted_from_graph => remove crates failing the filter *before* inverting edges
        if let Some(filter) = &config.filter_fn() {
            if *config.remove_unwanted_from_graph() {
                let mut to_remove: Vec<_> = graph
                    .node_indices()
                    .filter(|&idx| !(filter)(&graph[idx]))
                    .collect();
                to_remove.sort_by_key(|i| i.index());
                to_remove.reverse();
                for idx in to_remove {
                    graph.remove_node(idx);
                }
            }
        }

        // 3) Invert edges so that "if crateX depends on crateY" => we store Y->X
        {
            use petgraph::graph::EdgeIndex;
            let mut edges_to_invert = Vec::new();
            for eidx in graph.edge_indices() {
                if let Some((source, target)) = graph.edge_endpoints(eidx) {
                    edges_to_invert.push((source, target));
                }
            }
            for (source, target) in edges_to_invert {
                if let Some(weight) = graph.remove_edge(graph.find_edge(source, target).unwrap()) {
                    graph.add_edge(target, source, weight);
                }
            }
        }

        // 4) find focus node
        let focus_idx = match graph.node_indices().find(|&idx| graph[idx] == focus_crate_name) {
            Some(ix) => ix,
            None => {
                warn!("Focus crate '{focus_crate_name}' not found => empty result");
                return Ok(vec![]);
            }
        };

        // 5) gather ancestors via BFS over "incoming" edges
        let mut visited = HashSet::new();
        visited.insert(focus_idx);
        let mut stack = vec![focus_idx];
        while let Some(cur) = stack.pop() {
            for pred in graph.neighbors_directed(cur, Incoming) {
                if !visited.contains(&pred) {
                    visited.insert(pred);
                    stack.push(pred);
                }
            }
        }

        // 6) layering or normal
        if *config.layering_enabled() {
            // layering => flatten
            let flat = {
                let mut sub_layers = layered_subgraph_internal(&mut graph, &visited, config).await?;
                let mut tmp = Vec::new();
                for layer in sub_layers.drain(..) {
                    tmp.extend(layer);
                }
                tmp
            };
            Ok(flat)
        } else {
            // normal toposort
            match toposort(&graph, None) {
                Ok(sorted) => {
                    let mut result = Vec::new();
                    for idx in sorted {
                        // skip if not visited
                        if !visited.contains(&idx) {
                            continue;
                        }
                        // if remove_unwanted_from_graph=false, we do final filter check:
                        if let Some(filter) = &config.filter_fn() {
                            let nm = &graph[idx];
                            if !*config.remove_unwanted_from_graph() && !(filter)(nm) {
                                continue;
                            }
                        }
                        result.push(graph[idx].clone());
                    }
                    if *config.reverse_order() {
                        result.reverse();
                    }
                    Ok(result)
                }
                Err(cycle) => {
                    let node_id = cycle.node_id();
                    error!("Cycle in partial => node={:?}", node_id);
                    Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                        cycle_node_id: node_id,
                    })
                }
            }
        }
    }

    async fn layered_topological_order_upto_crate(
        &self,
        config: &TopologicalSortConfig,
        focus_crate_name: &str
    ) -> Result<Vec<Vec<String>>, WorkspaceError> {
        trace!("Workspace::layered_topological_order_upto_crate - focus={focus_crate_name}");

        // 1) cargo graph, handle empty/cycle
        let mut graph = match self.generate_dependency_tree().await {
            Ok(g) => g,
            Err(e) => {
                if let WorkspaceError::CargoMetadataError(CargoMetadataError::MetadataError { error: ref meta_err }) = e {
                    let stderr = format!("{:?}", meta_err);
                    if stderr.contains("cyclic package dependency") {
                        error!("Cargo says cycle => mapping to CycleDetectedInWorkspaceDependencyGraph");
                        return Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                            cycle_node_id: NodeIndex::new(0),
                        });
                    }
                    if stderr.contains("contains no package: The manifest is virtual, and the workspace has no members") {
                        warn!("Empty workspace => returning []");
                        return Ok(vec![]);
                    }
                }
                return Err(e);
            }
        };

        // 2) remove filter-fail nodes early if remove_unwanted_from_graph=true
        if let Some(filter) = &config.filter_fn() {
            if *config.remove_unwanted_from_graph() {
                let mut to_remove: Vec<_> = graph
                    .node_indices()
                    .filter(|&idx| !(filter)(&graph[idx]))
                    .collect();
                to_remove.sort_by_key(|i| i.index());
                to_remove.reverse();
                for idx in to_remove {
                    graph.remove_node(idx);
                }
            }
        }

        // 3) invert edges
        {
            use petgraph::graph::EdgeIndex;
            let mut edges_to_invert = Vec::new();
            for eidx in graph.edge_indices() {
                if let Some((source, target)) = graph.edge_endpoints(eidx) {
                    edges_to_invert.push((source, target));
                }
            }
            for (source, target) in edges_to_invert {
                if let Some(weight) = graph.remove_edge(graph.find_edge(source, target).unwrap()) {
                    graph.add_edge(target, source, weight);
                }
            }
        }

        // 4) find focus
        let focus_idx = match graph.node_indices().find(|&ix| graph[ix] == focus_crate_name) {
            Some(ix) => ix,
            None => {
                warn!("Focus crate '{focus_crate_name}' not found => empty layering");
                return Ok(vec![]);
            }
        };

        // 5) gather ancestors
        let mut visited = HashSet::new();
        visited.insert(focus_idx);
        let mut stack = vec![focus_idx];
        while let Some(cur) = stack.pop() {
            for pred in graph.neighbors_directed(cur, Incoming) {
                if !visited.contains(&pred) {
                    visited.insert(pred);
                    stack.push(pred);
                }
            }
        }

        // 6) layering
        let mut layers = layered_subgraph_internal(&mut graph, &visited, config).await?;
        if *config.reverse_order() {
            layers.reverse();
        }
        Ok(layers)
    }
}

#[cfg(test)]
mod test_focus_crate_topological_sort_exhaustive {
    use super::*;

    /// A helper to create a multi-crate workspace, optionally add dependencies,
    /// and then call `workspace.topological_order_upto_crate(...)`.
    /// Returns the resulting `Vec<String>`.
    async fn run_topo_upto_crate<H>(
        crate_configs: Vec<CrateConfig>,
        deps_map: &HashMap<&str, Vec<&str>>,  // crate => list of local deps
        focus_crate_name: &str,
        config: &TopologicalSortConfig,
    ) -> Result<Vec<String>, WorkspaceError>
    where
        for<'a> H: CrateHandleInterface<PathBuf> + Send + Sync + 'static,
        Workspace<PathBuf, H>: FocusCrateTopologicalSort + GenerateDependencyTree<Tree = Graph<String, ()>, Error = WorkspaceError>,
    {
        trace!("run_topo_upto_crate: start; focus={focus_crate_name}");
        let tmp_ws = create_mock_workspace(crate_configs)
            .await
            .map_err(|e| {
                error!("Failed creating mock workspace: {:?}", e);
                e
            })?;

        // For each crate => patch the local deps in cargo toml
        for (&crate_name, dep_list) in deps_map {
            let crate_path = tmp_ws.join(crate_name);
            let cargo_path = crate_path.join("Cargo.toml");
            let orig = fs::read_to_string(&cargo_path).await
                .unwrap_or_else(|_| panic!("Unable to read Cargo.toml for crate={crate_name}"));

            let mut new_section = String::new();
            if !dep_list.is_empty() {
                new_section.push_str("\n[dependencies]\n");
                let mut seen = HashSet::new();
                for dep in dep_list {
                    if seen.insert(*dep) {
                        new_section.push_str(&format!(
                            r#"{dep} = {{ path = "../{dep}" }}
"#
                        ));
                    }
                }
            }
            let new_toml = format!("{orig}\n{new_section}");
            fs::write(&cargo_path, new_toml).await
                .unwrap_or_else(|_| panic!("Failed to write updated Cargo.toml for crate={crate_name}"));
        }

        // Now parse the workspace
        let workspace = Workspace::<PathBuf, H>::new(&tmp_ws).await?;
        // Then call the trait method
        workspace.topological_order_upto_crate(config, focus_crate_name).await
    }

    /// Similar helper for `layered_topological_order_upto_crate`.
    async fn run_layered_upto_crate<H>(
        crate_configs: Vec<CrateConfig>,
        deps_map: &HashMap<&str, Vec<&str>>,
        focus_crate_name: &str,
        config: &TopologicalSortConfig,
    ) -> Result<Vec<Vec<String>>, WorkspaceError>
    where
        for<'a> H: CrateHandleInterface<PathBuf> + Send + Sync + 'static,
        Workspace<PathBuf, H>: FocusCrateTopologicalSort + GenerateDependencyTree<Tree = Graph<String, ()>, Error = WorkspaceError>,
    {
        trace!("run_layered_upto_crate: start; focus={focus_crate_name}");
        let tmp_ws = create_mock_workspace(crate_configs).await?;
        // Insert dependencies
        for (&crate_name, dep_list) in deps_map {
            let crate_path = tmp_ws.join(crate_name);
            let cargo_path = crate_path.join("Cargo.toml");
            let orig = fs::read_to_string(&cargo_path).await
                .unwrap_or_else(|_| panic!("Unable to read Cargo.toml for crate={crate_name}"));

            let mut new_section = String::new();
            if !dep_list.is_empty() {
                new_section.push_str("\n[dependencies]\n");
                let mut seen = HashSet::new();
                for dep in dep_list {
                    if seen.insert(*dep) {
                        new_section.push_str(&format!(
                            r#"{dep} = {{ path = "../{dep}" }}
"#
                        ));
                    }
                }
            }
            let new_toml = format!("{orig}\n{new_section}");
            fs::write(&cargo_path, new_toml).await
                .unwrap_or_else(|_| panic!("Failed to write updated Cargo.toml for crate={crate_name}"));
        }

        let workspace = Workspace::<PathBuf, H>::new(&tmp_ws).await?;
        workspace.layered_topological_order_upto_crate(config, focus_crate_name).await
    }

    // ---------------------------------------------------------------------------
    // Test 1) Focus crate not present => empty result
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_focus_crate_not_found() {
        trace!("test_focus_crate_not_found - start");
        let crateA = CrateConfig::new("crateA").with_src_files();
        let crateB = CrateConfig::new("crateB").with_src_files();

        let config = TopologicalSortConfigBuilder::default().build().unwrap();
        let result = run_topo_upto_crate::<CrateHandle>(
            vec![crateA, crateB],
            &HashMap::new(),
            "nonExistent",
            &config,
        ).await
          .expect("Should not error => just empty partial result");
        assert!(result.is_empty(), "No focus => empty");
        info!("test_focus_crate_not_found => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 2) Focus crate has no incoming edges => partial => just that crate
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_focus_crate_no_incoming() {
        trace!("test_focus_crate_no_incoming - start");
        let crateX = CrateConfig::new("crateX").with_src_files();
        let crateY = CrateConfig::new("crateY").with_src_files();
        // We'll define Y depends on X => cargo sees Y->X => after inverts, X->Y => so X has no incoming edges
        // If we pick focus=X => partial => just [X].
        let mut deps_map = HashMap::new();
        deps_map.insert("crateY", vec!["crateX"]);

        let config = TopologicalSortConfigBuilder::default().build().unwrap();
        let result = run_topo_upto_crate::<CrateHandle>(
            vec![crateX, crateY],
            &deps_map,
            "crateX",
            &config,
        ).await
          .expect("No cycle => partial subgraph => crateX alone");
        assert_eq!(result, vec!["crateX"]);
        info!("test_focus_crate_no_incoming => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 3) Basic chain: focus on top => gather ancestors
    //
    //   We'll define chain: A->B->C => cargo sees B->A, C->B => final => A->B->C.
    //   Then focus=C => partial => [A, B, C] in topological order => [A, B, C].
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_focus_crate_chain() {
        trace!("test_focus_crate_chain - start");
        let crateA = CrateConfig::new("A").with_src_files();
        let crateB = CrateConfig::new("B").with_src_files();
        let crateC = CrateConfig::new("C").with_src_files();
        // B => A => B->A => final => A->B
        // C => B => C->B => final => B->C => chain => A->B->C
        let mut deps_map = HashMap::new();
        deps_map.insert("B", vec!["A"]);
        deps_map.insert("C", vec!["B"]);

        let config = TopologicalSortConfigBuilder::default().build().unwrap();
        let result = run_topo_upto_crate::<CrateHandle>(
            vec![crateA, crateB, crateC],
            &deps_map,
            "C",
            &config,
        ).await
          .expect("No cycle => partial subgraph => [A,B,C]");
        // We want topological => [A, B, C]
        assert_eq!(result, vec!["A","B","C"]);
        info!("test_focus_crate_chain => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 4) cycle in relevant subgraph => fail
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_focus_crate_cycle() {
        trace!("test_focus_crate_cycle - start");
        // We'll define cycX => cycY => cycZ => cycX => cycle
        // Then we pick focus=cycZ => partial => includes cycX, cycY => cycle => error
        let cycX = CrateConfig::new("cycX").with_src_files();
        let cycY = CrateConfig::new("cycY").with_src_files();
        let cycZ = CrateConfig::new("cycZ").with_src_files();

        // Y => X => cargo sees Y->X => final => X->Y
        // Z => Y => cargo sees Z->Y => final => Y->Z
        // X => Z => cargo sees X->Z => final => Z->X => cycle among all X,Y,Z
        let mut deps_map = HashMap::new();
        deps_map.insert("cycY", vec!["cycX"]);
        deps_map.insert("cycZ", vec!["cycY"]);
        deps_map.insert("cycX", vec!["cycZ"]);

        let config = TopologicalSortConfigBuilder::default().build().unwrap();
        let result = run_topo_upto_crate::<CrateHandle>(
            vec![cycX, cycY, cycZ],
            &deps_map,
            "cycZ",
            &config,
        ).await;
        assert!(result.is_err(), "Cycle => partial subgraph => error");
        match result {
            Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {..}) => {},
            other => panic!("Expected cycle error, got {other:?}"),
        }
        info!("test_focus_crate_cycle => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 5) filter skip => remove_unwanted_from_graph=false => skip from final
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_focus_filter_skip() {
        trace!("test_focus_filter_skip - start");
        // We'll define crates: A->B->C => chain => [A, B, C].
        // Focus on C => partial => [A, B, C].
        // Then filter out B => skip B from final => [A, C].
        let cA = CrateConfig::new("A").with_src_files();
        let cB = CrateConfig::new("B").with_src_files();
        let cC = CrateConfig::new("C").with_src_files();
        let mut deps_map = HashMap::new();
        deps_map.insert("B", vec!["A"]);
        deps_map.insert("C", vec!["B"]);

        let mut filter_fn: Arc<dyn Fn(&str)->bool + Send + Sync> = Arc::new(|nm: &str| nm != "B");
        let config = TopologicalSortConfigBuilder::default()
            .filter_fn(Some(filter_fn))
            .remove_unwanted_from_graph(false)
            .build()
            .unwrap();

        let result = run_topo_upto_crate::<CrateHandle>(
            vec![cA, cB, cC],
            &deps_map,
            "C",
            &config,
        ).await
          .expect("No cycle => partial => skip B in final");
        // So the BFS partial subgraph includes A,B,C, but we skip B in final => [A, C].
        let mut sorted = result.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["A","C"]);
        info!("test_focus_filter_skip => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 6) filter remove => remove B from subgraph => we might lose path from A->C if that was required
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_focus_filter_remove() {
        trace!("test_focus_filter_remove - start");
        // A->B->C => if we remove B from the subgraph, that means C has no path from A. But let's see if partial is [C] only or fails.
        // Actually, we gather ancestors of C => that includes B, A. Then we remove B from the graph => does that isolate A? Possibly yes, so the code might produce [A, C] if there's a direct A->C.
        // We'll do a scenario: B => A, C => B => final => A->B->C. Then remove B => that leaves C alone if there's no direct A->C.
        // So final partial => [C].
        let cA = CrateConfig::new("A").with_src_files();
        let cB = CrateConfig::new("B").with_src_files();
        let cC = CrateConfig::new("C").with_src_files();
        let mut deps_map = HashMap::new();
        deps_map.insert("B", vec!["A"]);
        deps_map.insert("C", vec!["B"]);

        let filter_fn: Arc<dyn Fn(&str)->bool + Send + Sync> = Arc::new(|nm: &str| nm != "B");
        let config = TopologicalSortConfigBuilder::default()
            .filter_fn(Some(filter_fn))
            .remove_unwanted_from_graph(true)
            .build()
            .unwrap();

        let result = run_topo_upto_crate::<CrateHandle>(
            vec![cA, cB, cC],
            &deps_map,
            "C",
            &config,
        ).await
          .expect("No cycle => partial => remove B => leftover might be [C]");
        // Because B is removed => no path from A to C => only focus crate remains => [C]
        assert_eq!(result, vec!["C"]);
        info!("test_focus_filter_remove => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 7) weighting => reorder layering => we do layering => flatten => check if weighting
    //         changes the order among the same layer
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_focus_weighting_layered_flat() {
        trace!("test_focus_weighting_layered_flat - start");
        // We'll define crates X, Y, Z => Y->X, Z->X => X has no incoming edges. Focus on X => partial => just X by itself, or we test something else?
        // Instead let's do a scenario: focus on D => ancestors => B, C => B->A, C->A => layering => first layer => [B,C], second => [D]? Actually let's be explicit.

        // We'll define: B => A, C => A => D => B, D => C => so if focus=D, we gather B,C,A => layering => [B, C],[D], ignoring A if there's no direct path? Actually B->A => we gather A as well. Let's see the code:

        // B => A => cargo sees B->A => final => A->B
        // C => A => cargo sees C->A => final => A->C
        // D => B, C => cargo sees D->B, D->C => final => B->D, C->D => So focusing on D => partial => [A,B,C,D]. layering => [A],[B,C],[D]. We'll do weighting => B=10, C=5 => so that in the layer with B,C, we see [C,B].
        let cA = CrateConfig::new("A").with_src_files();
        let cB = CrateConfig::new("B").with_src_files();
        let cC = CrateConfig::new("C").with_src_files();
        let cD = CrateConfig::new("D").with_src_files();
        let mut deps_map = HashMap::new();
        deps_map.insert("B", vec!["A"]);
        deps_map.insert("C", vec!["A"]);
        deps_map.insert("D", vec!["B","C"]);  // cargo sees D->B, D->C => final => B->D, C->D => partial ancestors => [A,B,C,D]

        let weighting: Arc<dyn Fn(&str)->u32 + Send + Sync> = Arc::new(|nm: &str| match nm {
            "B" => 10,
            "C" => 5,
            _ => 1,
        });
        let mut config = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .weighting_fn(Some(weighting))
            .build()
            .unwrap();

        let result = run_topo_upto_crate::<CrateHandle>(
            vec![cA, cB, cC, cD],
            &deps_map,
            "D",
            &config,
        ).await
          .expect("No cycle => partial => layering => flatten => check weighting in the middle layer");
        // layering => [A],[C,B],[D] by weighting => flatten => [A,C,B,D]
        assert_eq!(result, vec!["A","C","B","D"]);
        info!("test_focus_weighting_layered_flat => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 8) reverse => invert final order
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_focus_reverse_order() {
        trace!("test_focus_reverse_order - start");
        // We'll define chain: A->B->C => focus=C => partial => [A,B,C]. Then reversed => [C,B,A].
        let cA = CrateConfig::new("A").with_src_files();
        let cB = CrateConfig::new("B").with_src_files();
        let cC = CrateConfig::new("C").with_src_files();
        let mut deps_map = HashMap::new();
        deps_map.insert("B", vec!["A"]);
        deps_map.insert("C", vec!["B"]);

        // normal
        let cfg_normal = TopologicalSortConfigBuilder::default()
            .build()
            .unwrap();
        let normal = run_topo_upto_crate::<CrateHandle>(
            vec![cA.clone(), cB.clone(), cC.clone()],
            &deps_map,
            "C",
            &cfg_normal,
        ).await
          .expect("chain => partial => [A,B,C]");
        assert_eq!(normal, vec!["A","B","C"]);

        // reversed
        let cfg_rev = TopologicalSortConfigBuilder::default()
            .reverse_order(true)
            .build()
            .unwrap();
        let reversed = run_topo_upto_crate::<CrateHandle>(
            vec![cA, cB, cC],
            &deps_map,
            "C",
            &cfg_rev,
        ).await
          .expect("chain => partial => reversed => [C,B,A]");
        assert_eq!(reversed, vec!["C","B","A"]);
        info!("test_focus_reverse_order => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 9) layered_topological_order_upto_crate => focus not found => empty layering
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_layered_focus_not_found() {
        trace!("test_layered_focus_not_found - start");
        let crateA = CrateConfig::new("A").with_src_files();
        let crateB = CrateConfig::new("B").with_src_files();
        let config = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .build()
            .unwrap();

        let layers = run_layered_upto_crate::<CrateHandle>(
            vec![crateA, crateB],
            &HashMap::new(),
            "nonExist",
            &config,
        ).await
          .expect("Focus not found => empty layering");
        assert!(layers.is_empty(), "Should yield empty");
        info!("test_layered_focus_not_found => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 10) layered => small chain => focus => layering
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_layered_focus_chain() {
        trace!("test_layered_focus_chain - start");
        // A->B->C => focus=C => partial => layering => e.g. [A],[B],[C].
        let crateA = CrateConfig::new("A").with_src_files();
        let crateB = CrateConfig::new("B").with_src_files();
        let crateC = CrateConfig::new("C").with_src_files();
        let mut deps_map = HashMap::new();
        deps_map.insert("B", vec!["A"]);
        deps_map.insert("C", vec!["B"]);

        let config = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .build()
            .unwrap();
        let layers = run_layered_upto_crate::<CrateHandle>(
            vec![crateA, crateB, crateC],
            &deps_map,
            "C",
            &config,
        ).await
          .expect("Should yield layered approach => [A],[B],[C].");
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec!["A"]);
        assert_eq!(layers[1], vec!["B"]);
        assert_eq!(layers[2], vec!["C"]);
        info!("test_layered_focus_chain => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 11) layered => weighting => focus => flatten => we confirm the layering’s order
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_layered_focus_weighting() {
        trace!("test_layered_focus_weighting - start");
        // We'll define: B => A, C => A, D => B, D => C => focusing on D => partial => [A,B,C,D]
        // layering => [A],[B,C],[D]. We'll do weighting => B=20, C=10 => so second layer => [C,B].
        let cA = CrateConfig::new("A").with_src_files();
        let cB = CrateConfig::new("B").with_src_files();
        let cC = CrateConfig::new("C").with_src_files();
        let cD = CrateConfig::new("D").with_src_files();
        let mut deps_map = HashMap::new();
        deps_map.insert("B", vec!["A"]);
        deps_map.insert("C", vec!["A"]);
        deps_map.insert("D", vec!["B","C"]);

        let weighting: Arc<dyn Fn(&str)->u32 + Send + Sync> = Arc::new(|nm: &str| match nm {
            "B" => 20,
            "C" => 10,
            _ => 1,
        });
        let config = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .weighting_fn(Some(weighting))
            .build()
            .unwrap();

        let layers = run_layered_upto_crate::<CrateHandle>(
            vec![cA, cB, cC, cD],
            &deps_map,
            "D",
            &config,
        ).await
          .expect("No cycle => partial => layered => check weighting order in layer2");
        // We expect 3 layers => [A], [C,B], [D]
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec!["A"]);
        assert_eq!(layers[2], vec!["D"]);
        // The middle layer => [C,B]
        assert_eq!(layers[1], vec!["C","B"]);
        info!("test_layered_focus_weighting => passed");
    }

    // ---------------------------------------------------------------------------
    // Test 12) layered => reverse => e.g. chain => focus => normal => [A],[B],[C], reversed => [C],[B],[A]
    // ---------------------------------------------------------------------------
    #[traced_test]
    async fn test_layered_focus_reverse() {
        trace!("test_layered_focus_reverse - start");
        // A->B->C => focus=C => layering => [A],[B],[C]. reversed => [C],[B],[A]
        let crateA = CrateConfig::new("A").with_src_files();
        let crateB = CrateConfig::new("B").with_src_files();
        let crateC = CrateConfig::new("C").with_src_files();
        let mut deps_map = HashMap::new();
        deps_map.insert("B", vec!["A"]);
        deps_map.insert("C", vec!["B"]);

        let cfg = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .build()
            .unwrap();
        let normal = run_layered_upto_crate::<CrateHandle>(
            vec![crateA.clone(), crateB.clone(), crateC.clone()],
            &deps_map,
            "C",
            &cfg,
        ).await
          .expect("Should be [ [A],[B],[C] ]");
        assert_eq!(normal.len(), 3);
        assert_eq!(normal[0], vec!["A"]);
        assert_eq!(normal[1], vec!["B"]);
        assert_eq!(normal[2], vec!["C"]);

        let rev_cfg = TopologicalSortConfigBuilder::default()
            .layering_enabled(true)
            .reverse_order(true)
            .build()
            .unwrap();
        let reversed = run_layered_upto_crate::<CrateHandle>(
            vec![crateA, crateB, crateC],
            &deps_map,
            "C",
            &rev_cfg,
        ).await
          .expect("reversed => [ [C],[B],[A] ]");
        assert_eq!(reversed.len(), 3);
        assert_eq!(reversed[0], vec!["C"]);
        assert_eq!(reversed[1], vec!["B"]);
        assert_eq!(reversed[2], vec!["A"]);
        info!("test_layered_focus_reverse => passed");
    }
}
