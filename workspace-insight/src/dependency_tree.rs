crate::ix!();

type DependencyGraph = DiGraphMap<String, ()>;

impl Workspace {
    /// Generates a dependency tree for all crates in the workspace.
    pub async fn generate_dependency_tree(&self) -> Result<DependencyGraph, WorkspaceError> {
        // Use cargo_metadata to get the metadata
        let metadata = self.get_cargo_metadata().await?;

        // Create a directed graph
        let mut graph = DependencyGraph::new();

        // Map package IDs to package names for easy lookup
        let mut id_to_name: HashMap<PackageId, String> = HashMap::new();
        for package in &metadata.packages {
            id_to_name.insert(package.id.clone(), package.name.clone());
        }

        // Add nodes and edges to the graph
        if let Some(resolve) = &metadata.resolve {
            for node in &resolve.nodes {
                let package_name = id_to_name.get(&node.id).unwrap().clone();
                graph.add_node(package_name.clone());

                for dep in &node.deps {
                    if let Some(dep_name) = id_to_name.get(&dep.pkg) {
                        graph.add_edge(package_name.clone(), dep_name.clone(), ());
                    }
                }
            }
        }

        Ok(graph)
    }

    /// Helper method to get cargo metadata asynchronously
    async fn get_cargo_metadata(&self) -> Result<Metadata, WorkspaceError> {
        let path = self.path().to_path_buf();
        let metadata = task::spawn_blocking(move || {
            MetadataCommand::new()
                .current_dir(&path)
                .exec()
                .map_err(|e| WorkspaceError::CargoMetadataError(e.to_string()))
        })
        .await
        .map_err(|e| WorkspaceError::CargoMetadataError(e.to_string()))??;
        Ok(metadata)
    }

    /// Generates the dependency tree and returns it in DOT format.
    pub async fn generate_dependency_tree_dot(&self) -> Result<String, WorkspaceError> {
        let graph = self.generate_dependency_tree().await?;
        let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
        Ok(format!("{:?}", dot))
    }
}

