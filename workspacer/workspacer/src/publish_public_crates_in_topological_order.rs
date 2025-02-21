// ---------------- [ File: src/publish_public_crates_in_topological_order.rs ]
crate::ix!();

#[async_trait]
impl<P, H: CrateHandleInterface<P>> TryPublish for Workspace<P, H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    type Error = WorkspaceError;

    /// Publishes all public crates in a workspace in topological order,
    /// skipping those that are already on crates.io or that produce an
    /// "already exists" error during publish.
    async fn try_publish(&self) -> Result<(), Self::Error> {
        info!("Gathering dependency graph in topological order...");

        // Generate the dependency graph:
        let dependency_graph = self.generate_dependency_tree().await?;

        // Convert the graph to a topological order of node indices:
        let topo_order = petgraph::algo::toposort(&dependency_graph, None).map_err(|cycle| {
            WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                cycle_node_id: cycle.node_id(),
            }
        })?;

        // Build a map of `crate_name -> &CrateHandle`
        let mut name_to_handle = BTreeMap::<String, &H>::new();
        for crate_handle in self.into_iter() {
            let package_section = crate_handle
                .cargo_toml()
                .get_package_section()
                .map_err(|e| WorkspaceError::CrateError(CrateError::CargoTomlError(e)))?;
            let crate_name = package_section
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown_name>")
                .to_string();

            name_to_handle.insert(crate_name, crate_handle);
        }

        info!(
            "Topologically sorted crate list has length: {}",
            topo_order.len()
        );

        // For each node index in topological order, retrieve the crate name from the node weight
        for node_index in topo_order {
            // `node_weight` gives us the string we inserted in the graph
            let crate_node_name = dependency_graph
                .node_weight(node_index)
                .expect("Graph node weight not found");

            // Then look up the handle by that string
            if let Some(crate_handle) = name_to_handle.get(crate_node_name) {
                let is_private = crate_handle.is_private()?;
                if is_private {
                    debug!("SKIP: crate '{crate_node_name}' is private.");
                    continue;
                }

                let crate_name    = crate_handle.name();
                let crate_version = crate_handle.version().expect("expected crate to have a version");

                info!("------------------------------------------------");
                info!("Crate:   {crate_name}");
                info!("Version: {crate_version}");

                // Check if published on crates.io
                if is_crate_version_published_on_crates_io(&crate_name, &crate_version).await? {
                    info!("SKIP: {crate_name}@{crate_version} is already on crates.io");
                } else {
                    info!("Attempting to publish {crate_name}@{crate_version} ...");
                    // Run cargo publish, ignoring 'already exists' errors
                    match crate_handle.try_publish().await {
                        Ok(_) => { /* all good */ }
                        Err(e) => {
                            error!("FATAL: Could not publish {crate_name}@{crate_version}.");
                            return Err(e.into());
                        }
                    };
                }
            }
        }

        info!("Done! All crates either published or skipped.");
        Ok(())
    }
}
