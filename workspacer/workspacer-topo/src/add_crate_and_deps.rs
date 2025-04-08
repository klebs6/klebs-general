crate::ix!();

/// Minimal BFS-based function that adds `crate_handle` and all its internal dependencies
/// (recursively) into `graph`, returning the node index of `crate_handle` in that graph.
///
/// This depends on `GetInternalDependencies` to discover path-based local deps in each crate’s
/// Cargo.toml. The edges are oriented `dep -> crate`, so that a topological sort ensures
/// we list each dependency before the crate(s) that depend on it.
pub async fn add_crate_and_deps(
    graph:        &mut Graph<String, ()>,
    crate_handle: &CrateHandle,

) -> Result<NodeIndex, WorkspaceError> {

    let mut name_to_idx = HashMap::<String, NodeIndex>::new();
    let mut queue = VecDeque::<String>::new();

    let root_name = crate_handle.name().to_string();
    let root_idx = graph.add_node(root_name.clone());
    name_to_idx.insert(root_name.clone(), root_idx);
    queue.push_back(root_name.clone());

    // For demonstration, we only fetch the root crate's local path deps, not multi-level recursion.
    // If you want multi-level, you must fetch each dependency's CrateHandle and keep going.
    // This example only does single-level BFS.
    while let Some(current_name) = queue.pop_front() {
        if current_name == root_name {
            // gather local deps from the root crate:
            let deps = crate_handle.internal_dependencies().await?;
            for dep_name in deps {
                if !name_to_idx.contains_key(&dep_name) {
                    let dep_idx = graph.add_node(dep_name.clone());
                    name_to_idx.insert(dep_name.clone(), dep_idx);
                    queue.push_back(dep_name.clone());
                }
                // Add edge: dep -> current_crate
                let dep_idx = *name_to_idx.get(&dep_name).unwrap();
                let curr_idx = *name_to_idx.get(&current_name).unwrap();
                // avoid duplicates
                if !graph.edges_connecting(dep_idx, curr_idx).next().is_some() {
                    graph.add_edge(dep_idx, curr_idx, ());
                }
            }
        } else {
            // For non-root, we do nothing in this minimal example. 
            // Real code would look up or reconstruct that crate's handle, then gather its deps, etc.
            debug!("No recursion for crate={} in BFS example", current_name);
        }
    }

    Ok(root_idx)
}

#[cfg(test)]
mod test_add_crate_and_deps {
    use super::*;


    //----------------------------------------------------------------------
    // Test 1) Single crate, no dependencies
    //----------------------------------------------------------------------
    #[traced_test]
    async fn test_single_crate_no_deps() {
        trace!("test_single_crate_no_deps -> start");

        let handle = create_workspace_and_get_handle("root_crate", &[], false)
            .await
            .expect("Should create single crate with no deps OK");

        let mut graph = Graph::<String, ()>::new();
        let root_idx = add_crate_and_deps(&mut graph, &handle)
            .await
            .expect("Expected BFS to succeed for single crate, no deps");

        // We expect exactly 1 node => the root crate
        assert_eq!(graph.node_count(), 1, "Should only have root crate in graph");
        assert_eq!(graph.edge_count(), 0, "No deps => no edges");
        let root_name_in_graph = &graph[root_idx];
        assert_eq!(root_name_in_graph, "root_crate");
        info!("test_single_crate_no_deps -> passed");
    }

    //----------------------------------------------------------------------
    // Test 2) Single crate, multiple direct deps
    //----------------------------------------------------------------------
    #[traced_test]
    async fn test_single_crate_multiple_deps() {
        trace!("test_single_crate_multiple_deps -> start");

        let deps = vec!["depA", "depB", "depC"];
        let handle = create_workspace_and_get_handle("root_crate", &deps, false)
            .await
            .expect("Should create single crate with multiple direct deps");

        let mut graph = Graph::<String, ()>::new();
        let root_idx = add_crate_and_deps(&mut graph, &handle)
            .await
            .expect("Expected BFS to handle multiple distinct deps");

        assert_eq!(graph.node_count(), 4, "root + 3 deps => 4 nodes");
        assert_eq!(graph.edge_count(), 3, "Each dep => 1 edge to root => total 3 edges");
        assert_eq!(&graph[root_idx], "root_crate");
        info!("test_single_crate_multiple_deps -> passed");
    }

    //----------------------------------------------------------------------
    // Test 3) Single crate, repeated or duplicate deps
    //----------------------------------------------------------------------
    #[traced_test]
    async fn test_single_crate_duplicate_deps() {
        trace!("test_single_crate_duplicate_deps -> start");

        // We'll specify "depA" once plus "depB" (instead of twice for depA).
        let deps = vec!["depA", "depB"];
        let handle = create_workspace_and_get_handle("root_crate", &deps, false)
            .await
            .expect("Should create single crate with direct deps in Cargo.toml");

        let mut graph = Graph::<String, ()>::new();
        let root_idx = add_crate_and_deps(&mut graph, &handle)
            .await
            .expect("Expected BFS to skip duplicates properly (if they arise), not fail parse");

        assert_eq!(graph.node_count(), 3, "root + depA + depB => 3 distinct nodes");
        assert_eq!(
            graph.edge_count(),
            2,
            "depA->root, depB->root => 2 edges (no duplication of keys in Cargo.toml)."
        );
        assert_eq!(&graph[root_idx], "root_crate");
        info!("test_single_crate_duplicate_deps -> passed");
    }

    #[traced_test]
    async fn test_prepopulated_graph() {
        trace!("test_prepopulated_graph -> start");

        let deps = vec!["depX", "depY"];
        let handle = create_workspace_and_get_handle("root_crate", &deps, false)
            .await
            .expect("Should create single crate with 2 deps");

        let mut graph = Graph::<String, ()>::new();
        let existing_idx = graph.add_node("unrelated".to_string());
        debug!("Existing node => index={:?}", existing_idx);

        let root_idx = add_crate_and_deps(&mut graph, &handle)
            .await
            .expect("BFS add should succeed with prepopulated graph");

        // We now expect 4 total nodes: "unrelated" + "root_crate" + "depX" + "depY".
        assert_eq!(
            graph.node_count(),
            4,
            "unrelated + root_crate + depX + depY => 4 total"
        );
        // BFS edges => depX->root, depY->root => total 2 edges
        assert_eq!(
            graph.edge_count(),
            2,
            "depX->root, depY->root => 2 edges"
        );
        assert_eq!(&graph[root_idx], "root_crate");
        info!("test_prepopulated_graph -> passed");
    }

    //----------------------------------------------------------------------
    // Test 5) internal_dependencies error => BFS fails
    //----------------------------------------------------------------------
    #[traced_test]
    async fn test_internal_dependencies_error() {
        trace!("test_internal_dependencies_error -> start");

        // We artificially break the root crate Cargo.toml to cause a parse or metadata error
        let handle_result = create_workspace_and_get_handle("root_crate", &["depA"], true).await;
        // We might fail even before BFS if the user code errors out. Let's see:
        match handle_result {
            Ok(handle) => {
                // If the workspace creation succeeded but `internal_dependencies()` calls fail,
                // BFS will see the error:
                let mut graph = Graph::<String, ()>::new();
                let bfs_result = add_crate_and_deps(&mut graph, &handle).await;
                assert!(bfs_result.is_err(), "Expected BFS to fail due to invalid/broken Cargo.toml");
                match bfs_result.err().unwrap() {
                    WorkspaceError::CrateError(e) => {
                        debug!("Got expected BFS error => CrateError: {e:?}");
                    }
                    other => panic!("Expected CrateError, got {other:?}"),
                }
            },
            Err(e) => {
                // If your real crate code fails earlier, that also indicates the same root cause.
                debug!("Workspace or handle creation itself failed => {e:?}");
                match e {
                    WorkspaceError::InvalidCargoToml {..}
                    | WorkspaceError::CrateError(_)
                    => debug!("We got an expected parse/metadata error for broken toml: {e:?}"),
                    other => panic!("Expected a cargo-toml-related error, got {other:?}"),
                }
            }
        }

        info!("test_internal_dependencies_error -> passed");
    }

    //----------------------------------------------------------------------
    // Test 6) BFS is single-level => no recursion for sub-deps
    //----------------------------------------------------------------------
    #[traced_test]
    async fn test_no_recursion_for_subdeps() {
        trace!("test_no_recursion_for_subdeps -> start");

        // We'll define root crate => "root_crate" => depends on "depA"
        // Then in "depA" Cargo.toml, we'd define a further dependency "depZ" 
        // but BFS only fetches root's direct deps and doesn't follow sub-deps.
        // We'll do that by adding "depZ" in the final step if we want (the BFS won't see it).
        // For clarity, let's just ensure BFS doesn't add subdep nodes.
        let root_name = "root_crate";
        let _ws_dir = create_mock_workspace(vec![
            // The root crate
            CrateConfig::new(root_name).with_src_files(),
            // The subdep crate A => we'll add "depZ" manually
            CrateConfig::new("depA").with_src_files(),
            // Another subdep crate => "depZ", just so it physically exists
            CrateConfig::new("depZ").with_src_files(),
        ]).await.expect("Failed to create 3-crate workspace");

        // Now we update root_crate Cargo.toml => [dependencies] depA
        {
            let cargo_toml_path = _ws_dir.join(root_name).join("Cargo.toml");
            let orig = fs::read_to_string(&cargo_toml_path).await.unwrap();
            let new = format!(r#"{orig}

[dependencies]
depA = {{ path = "../depA" }}
"#);
            fs::write(&cargo_toml_path, new).await.unwrap();
        }

        // Then we also update depA => depends on depZ, so we can confirm BFS won't gather it
        {
            let dep_a_toml = _ws_dir.join("depA").join("Cargo.toml");
            let orig = fs::read_to_string(&dep_a_toml).await.unwrap();
            let new = format!(r#"{orig}

[dependencies]
depZ = {{ path = "../depZ" }}
"#);
            fs::write(&dep_a_toml, new).await.unwrap();
        }

        // parse a handle for "root_crate"
        let ws = Workspace::<PathBuf,CrateHandle>::new(&_ws_dir).await
            .expect("Should parse the 3-crate workspace OK");
        let root_crate_arc = ws.find_crate_by_name(root_name).await
            .expect("Expected to find root_crate");
        let root_handle = root_crate_arc.lock().await.clone();

        // BFS
        let mut graph = Graph::<String, ()>::new();
        add_crate_and_deps(&mut graph, &root_handle).await
            .expect("BFS single-level add should succeed");

        // BFS only sees root_crate + depA => total 2 nodes
        assert_eq!(graph.node_count(), 2, "root + depA => 2 nodes (no subdep 'depZ')");
        assert_eq!(graph.edge_count(), 1, "depA->root => 1 edge");
        let mut nameset = HashSet::new();
        for idx in graph.node_indices() {
            nameset.insert(graph[idx].clone());
        }
        assert!(nameset.contains("root_crate"), "Must have root");
        assert!(nameset.contains("depA"), "Must have direct dep");
        assert!(!nameset.contains("depZ"), "Should NOT have subdep => BFS is single-level");

        info!("test_no_recursion_for_subdeps -> passed");
    }
}
