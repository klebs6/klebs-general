// ---------------- [ File: src/dependency_tree.rs ]
crate::ix!();

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

        println!("dependency tree: {:#?}", graph);

        Ok(graph)
    }

    /// Generates the dependency tree and returns it in DOT format.
    async fn generate_dependency_tree_dot(&self) -> Result<String, WorkspaceError> {
        let graph = self.generate_dependency_tree().await?;
        let dot = Dot::with_config(&graph, &[DotConfig::EdgeNoLabel]);
        Ok(format!("{:?}", dot))
    }
}
