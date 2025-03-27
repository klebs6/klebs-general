crate::ix!();

#[async_trait]
impl<P, H> TryPublish for Workspace<P, H>
where
    P: From<std::path::PathBuf> + AsRef<std::path::Path> + Send + Sync + 'static,
    H: TryPublish<Error = CrateError> + CrateHandleInterface<P> + Send + Sync + 'static,
{
    type Error = WorkspaceError;

    async fn try_publish(&self, dry_run: bool) -> Result<(), Self::Error> {
        tracing::info!("Gathering dependency graph in topological order...");

        let dependency_graph = self.generate_dependency_tree().await?;

        let topo_order = petgraph::algo::toposort(&dependency_graph, None)
            .map_err(|cycle| {
                WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                    cycle_node_id: cycle.node_id(),
                }
            })?;

        let mut name_to_handle = std::collections::BTreeMap::<String, std::sync::Arc<workspacer_3p::AsyncMutex<H>>>::new();
        for crate_handle in self.into_iter() {
            let guard = crate_handle.lock().await;
            let cargo_toml = guard.cargo_toml();
            let guard2 = cargo_toml.lock().await;
            let package_section = guard2.get_package_section()?;

            let crate_name = package_section
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("<unknown_name>")
                .to_string();

            name_to_handle.insert(crate_name, crate_handle.clone());
        }

        tracing::info!(
            "Topologically sorted crate list has length: {}",
            topo_order.len()
        );

        for node_index in topo_order {
            let crate_node_name = dependency_graph
                .node_weight(node_index)
                .expect("Graph node weight not found");

            if let Some(crate_handle) = name_to_handle.get(crate_node_name) {
                let guard = crate_handle.lock().await;
                let is_private = guard.is_private().await?;
                if is_private {
                    tracing::debug!("SKIP: crate '{}' is private.", crate_node_name);
                    continue;
                }

                let crate_name    = guard.name();
                let crate_version = guard.version()?;

                tracing::info!("------------------------------------------------");
                tracing::info!("Crate:   {}", crate_name);
                tracing::info!("Version: {}", crate_version);

                // Check if published
                if is_crate_version_published_on_crates_io(&crate_name, &crate_version).await? {
                    tracing::info!("SKIP: {}@{} is already on crates.io", crate_name, crate_version);
                } else {
                    tracing::info!("Attempting to publish {}@{} ...", crate_name, crate_version);
                    match guard.try_publish(dry_run).await {
                        Ok(_) => { /* success or 'already exists' skip */ }
                        Err(e) => {
                            tracing::error!("FATAL: Could not publish {}@{}.", crate_name, crate_version);
                            return Err(e.into());
                        }
                    }
                }
            }
        }

        tracing::info!("Done! All crates either published or skipped.");
        Ok(())
    }
}
