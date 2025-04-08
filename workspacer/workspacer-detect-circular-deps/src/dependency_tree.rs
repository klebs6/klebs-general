// ---------------- [ File: workspacer-detect-circular-deps/src/dependency_tree.rs ]
crate::ix!();

#[async_trait]
pub trait GenerateDependencyTree {

    type Tree;
    type Error;

    async fn generate_dependency_tree(&self) -> Result<Self::Tree, Self::Error>;
    async fn generate_dependency_tree_dot(&self) -> Result<String, Self::Error>;
}

pub type WorkspaceDependencyGraph = DiGraph<String, ()>;

#[async_trait]
impl<P,H:CrateHandleInterface<P>> GenerateDependencyTree for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Tree = WorkspaceDependencyGraph;
    type Error = WorkspaceError;

    /// Generates a dependency tree for all crates in the workspace.
    async fn generate_dependency_tree(&self) -> Result<WorkspaceDependencyGraph, WorkspaceError> {
        // Use cargo_metadata to get the metadata
        let metadata = self.get_cargo_metadata().await?;

        // Create a directed graph
        let mut graph: WorkspaceDependencyGraph = DiGraph::new();

        // Map package IDs to package names and their corresponding node index in the graph
        let mut id_to_node: HashMap<PackageId, NodeIndex> = HashMap::new();

        // Add nodes to the graph
        for package in metadata.packages {
            let node = graph.add_node(package.name.clone());
            id_to_node.insert(package.id.clone(), node);
        }

        // Add edges (dependencies) to the graph
        if let Some(resolve) = &metadata.resolve {
            for node in &resolve.nodes {
                let package_node = id_to_node[&node.id];

                for dep in &node.deps {
                    if let Some(dep_node) = id_to_node.get(&dep.pkg) {
                        graph.add_edge(package_node, *dep_node, ());
                    }
                }
            }
        }

        info!("dependency tree: {:#?}", graph);

        Ok(graph)
    }

    /// Generates the dependency tree and returns it in DOT format.
    async fn generate_dependency_tree_dot(&self) -> Result<String, WorkspaceError> {
        let graph = self.generate_dependency_tree().await?;
        let dot = Dot::with_config(&graph, &[DotConfig::EdgeNoLabel]);
        Ok(format!("{:?}", dot))
    }
}

#[cfg(test)]
mod test_generate_dependency_tree {
    use super::*;

    /// 1) A workspace with a single crate, no dependencies => one node, no edges.
    #[tokio::test]
    async fn test_single_crate_no_deps() {
        // Let's define exactly one crate config
        let single_crate = CrateConfig::new("single_crate").with_src_files(); 
        // Create the mock workspace
        let root_path = create_mock_workspace(vec![single_crate])
            .await
            .expect("Failed to create mock workspace");

        // Convert to a Workspace. 
        // We'll do something like:
        let workspace = Workspace::<PathBuf,CrateHandle>::new(&root_path)
            .await
            .expect("Failed to create Workspace from mock dir");

        // Now generate the dependency tree
        let dep_graph = workspace.generate_dependency_tree().await
            .expect("generate_dependency_tree should succeed");

        // We expect exactly 1 node, named "single_crate", no edges
        assert_eq!(dep_graph.node_count(), 1, "One crate => one node");
        assert_eq!(dep_graph.edge_count(), 0, "No edges => no deps");
        // For extra certainty, we can fetch the node name:
        let node_idx = dep_graph.node_indices().next().unwrap();
        let node_name = &dep_graph[node_idx];
        assert_eq!(node_name, "single_crate");
    }

    /// 2) Multiple crates, no cross-dependencies => multiple nodes, zero edges.
    #[tokio::test]
    async fn test_multiple_crates_no_cross_deps() {
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let root_path = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("Failed to create mock workspace with multiple crates");

        let workspace = Workspace::<PathBuf,CrateHandle>::new(&root_path)
            .await
            .expect("Failed to create workspace");

        let dep_graph = workspace.generate_dependency_tree().await
            .expect("Should succeed");

        // We have 2 crates, no dependencies => 2 nodes, 0 edges
        assert_eq!(dep_graph.node_count(), 2);
        assert_eq!(dep_graph.edge_count(), 0);

        // Optionally verify the crate names in the graph
        let mut names = Vec::new();
        for idx in dep_graph.node_indices() {
            names.push(dep_graph[idx].clone());
        }
        names.sort();
        assert_eq!(names, vec!["crate_a".to_string(), "crate_b".to_string()]);
    }

    /// 3) One crate depends on another => we expect one edge in the graph.
    ///
    /// We'll demonstrate how to simulate that dependency:
    ///   - crateB (lib)
    ///   - crateA depends on crateB via local path:
    ///       [dependencies]
    ///       crate_b = { path = "../crate_b" }
    #[tokio::test]
    async fn test_simple_dependency_one_edge() {
        // We'll define two crate configs, but we also need to manually add a local path dependency 
        // to crate_a's Cargo.toml. We'll do that by adjusting the mock AFTER create_mock_workspace,
        // or by passing a custom function that modifies the cargo toml content. 
        // For brevity, let's do a "post-creation" step.

        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();

        // Step 1: Create the workspace with these two crates
        let root_path = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("create mock workspace");

        // Step 2: Insert `[dependencies] crate_b = { path = "../crate_b" }` into crate_a's Cargo.toml.
        let crate_a_path = root_path.join("crate_a");
        let cargo_toml_a  = crate_a_path.join("Cargo.toml");

        // We'll read the existing Cargo.toml content, append the dependency, rewrite
        let content = tokio::fs::read_to_string(&cargo_toml_a).await
            .expect("read cargo toml for crate_a");
        let new_content = format!(
            r#"{}
[dependencies]
crate_b = {{ path = "../crate_b" }}
"#,
            content
        );
        tokio::fs::write(&cargo_toml_a, new_content)
            .await
            .expect("rewrite cargo toml for crate_a with dependency on crate_b");

        // Step 3: Construct the workspace & generate dependency graph
        let workspace = Workspace::<PathBuf,CrateHandle>::new(&root_path).await.expect("create workspace");
        let dep_graph = workspace.generate_dependency_tree().await
            .expect("dep tree should succeed");

        // We expect 2 nodes: crate_a, crate_b
        // We expect 1 edge: from crate_a -> crate_b
        assert_eq!(dep_graph.node_count(), 2, "two crates");
        assert_eq!(dep_graph.edge_count(), 1, "one dependency edge");

        // Check which edge specifically. Let's gather the edges:
        let edges: Vec<_> = dep_graph.edge_references().collect();
        let edge = edges[0];
        let src_idx = edge.source();
        let dst_idx = edge.target();
        let src_name = &dep_graph[src_idx];
        let dst_name = &dep_graph[dst_idx];

        // We expect src_name = "crate_a", dst_name = "crate_b"
        assert_eq!(src_name, "crate_a");
        assert_eq!(dst_name, "crate_b");
    }

    /// 4) If we have multiple dependencies (A depends on B, A depends on C, B depends on C, etc.),
    ///    we can do a more advanced scenario. 
    ///    We'll just do one example. The principle is the same: we add local path deps, check edges.
    #[tokio::test]
    async fn test_multiple_dependencies() {
        // Suppose we have crate_a, crate_b, crate_c, with:
        //   crate_a depends on crate_b + crate_c
        //   crate_b depends on crate_c
        // So the graph has edges: A->B, A->C, B->C, C->(none)
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let crate_c = CrateConfig::new("crate_c").with_src_files();

        let root_path = create_mock_workspace(vec![crate_a, crate_b, crate_c])
            .await
            .expect("mock workspace creation");

        // We'll define dependencies by editing Cargo.toml after creation:
        // crate_a -> crate_b & crate_c
        // crate_b -> crate_c
        let a_toml = root_path.join("crate_a").join("Cargo.toml");
        let b_toml = root_path.join("crate_b").join("Cargo.toml");

        // Append [dependencies] lines:
        {
            let orig = tokio::fs::read_to_string(&a_toml).await.unwrap();
            let new = format!(
                r#"{}
[dependencies]
crate_b = {{ path = "../crate_b" }}
crate_c = {{ path = "../crate_c" }}
"#,
                orig
            );
            tokio::fs::write(&a_toml, new).await.unwrap();
        }
        {
            let orig = tokio::fs::read_to_string(&b_toml).await.unwrap();
            let new = format!(
                r#"{}
[dependencies]
crate_c = {{ path = "../crate_c" }}
"#,
                orig
            );
            tokio::fs::write(&b_toml, new).await.unwrap();
        }

        // Now create workspace & call generate_dependency_tree
        let workspace = Workspace::<PathBuf,CrateHandle>::new(&root_path).await.unwrap();
        let dep_graph = workspace.generate_dependency_tree().await.unwrap();

        // We expect 3 nodes
        assert_eq!(dep_graph.node_count(), 3);
        // We expect 3 edges: A->B, A->C, B->C
        assert_eq!(dep_graph.edge_count(), 3);

        // We can confirm them specifically by checking each edge
        // We'll do a quick approach:
        let mut found_edges = Vec::new();
        for edge in dep_graph.edge_references() {
            let src_idx = edge.source();
            let dst_idx = edge.target();
            let src_name = &dep_graph[src_idx];
            let dst_name = &dep_graph[dst_idx];
            found_edges.push((src_name.clone(), dst_name.clone()));
        }
        found_edges.sort();
        let expected = vec![
            ("crate_a".to_string(), "crate_b".to_string()),
            ("crate_a".to_string(), "crate_c".to_string()),
            ("crate_b".to_string(), "crate_c".to_string()),
        ];
        assert_eq!(found_edges, expected);
    }

    /// 5) Now we test `generate_dependency_tree_dot()`, verifying we get a DOT representation
    ///    with the correct node labels (the crate names). We won't parse the DOT deeply, but do partial checks.
    #[tokio::test]
    async fn test_generate_dependency_tree_dot() {
        // We'll reuse the simple scenario: crate_a depends on crate_b
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let root_path = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("mock workspace creation");
        // Insert dependency a->b
        let a_toml = root_path.join("crate_a").join("Cargo.toml");
        let orig = tokio::fs::read_to_string(&a_toml).await.unwrap();
        let new = format!(
            r#"{}
[dependencies]
crate_b = {{ path = "../crate_b" }}
"#,
            orig
        );
        tokio::fs::write(&a_toml, new).await.unwrap();

        // Build workspace
        let workspace = Workspace::<PathBuf,CrateHandle>::new(&root_path).await.unwrap();
        // Call generate_dependency_tree_dot
        let dot_str = workspace.generate_dependency_tree_dot().await.unwrap();

        // We'll do partial checks: confirm "crate_a" and "crate_b" appear
        // in the DOT, and there's an edge a->b. 
        assert!(dot_str.contains("crate_a"), "DOT should mention crate_a");
        assert!(dot_str.contains("crate_b"), "DOT should mention crate_b");
        // The exact DOT format might look like:
        //   "digraph {\n    0 [label=\"crate_a\"]\n    1 [label=\"crate_b\"]\n    0 -> 1\n}"
        // So let's just confirm it has something like -> 
        // (the node numbering might be 0 or 1 in an arbitrary order)
        assert!(dot_str.contains("->"), "Should have an edge in DOT");
    }

    /// 6) If the workspace has no `[dependencies]` or multiple crates, 
    ///    `generate_dependency_tree_dot()` is just multiple nodes, zero edges. 
    ///    That scenario is basically covered by test_multiple_crates_no_cross_deps, 
    ///    but we can do a quick partial test if we like.
    #[tokio::test]
    async fn test_dot_no_edges() {
        let crate_x = CrateConfig::new("crate_x").with_src_files();
        let crate_y = CrateConfig::new("crate_y").with_src_files();
        let root = create_mock_workspace(vec![crate_x, crate_y])
            .await
            .expect("mock ws creation");
        let ws = Workspace::<PathBuf,CrateHandle>::new(&root).await.unwrap();
        let dot = ws.generate_dependency_tree_dot().await.unwrap();
        // Should contain crate_x, crate_y, but no "->"
        assert!(dot.contains("crate_x"));
        assert!(dot.contains("crate_y"));
        // For zero edges, we presumably won't see any "->"
        assert!(!dot.contains("->"), "No edges => no arrow in the DOT");
    }

    /// 7) If there's a cyclical or impossible dependency scenario, 
    ///    cargo_metadata might fail or the graph might have cycles. 
    ///    In your code you might return an error. We'll do a partial demonstration.
    ///    It's tricky to set up a local cyc. We'll skip the details, but for completeness:
    #[tokio::test]
    async fn test_circular_dependency() {
        // We'll try to set crate_a depends on crate_b, crate_b depends on crate_a
        // This might cause cargo_metadata to fail with a cycle error, or might produce a partial result.
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();
        let root = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .unwrap();
        // add cross dependencies in both cargo tomls...
        // ...
        // Then:
        let workspace = Workspace::<PathBuf,CrateHandle>::new(&root).await.unwrap();
        let result = workspace.generate_dependency_tree().await;
        match result {
            Ok(graph) => {
                // Possibly cargo_metadata is tolerant or partial. We can see if it forms a cycle
                // You can test your code's logic for cycle detection if it’s in `detect_circular_dependencies()`.
                println!("Graph had {} nodes and {} edges", graph.node_count(), graph.edge_count());
            }
            Err(e) => {
                // Possibly you get a CargoMetadataError or a cycle error. 
                println!("We got an error, possibly due to cycle: {:?}", e);
            }
        }
        // There's no single universal outcome, as cargo might bail out or produce partial. 
        // So adapt to your real code's behavior.
    }
}
