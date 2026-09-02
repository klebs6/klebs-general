crate::ix!();

/// Helper extension imported by `workspacer‑cli`.
#[async_trait]
pub trait TryPublishCrateTree<P, H>
where
    H: TryPublish<Error = CrateError> + CrateHandleInterface<P> + Send + Sync + 'static,
    P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static,
{
    /// Publish a *single* crate and every other workspace crate it directly
    /// or transitively depends on.  
    /// Crates are published in reverse topological order to match the
    /// existing `try_publish` behaviour.
    async fn try_publish_crate_tree(
        &self,
        root_crate_name: &str,
        dry_run: bool,
    ) -> Result<(), WorkspaceError>;
}

#[async_trait]
impl<P, H> TryPublishCrateTree<P, H> for Workspace<P, H>
where
    P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static,
    H: TryPublish<Error = CrateError>
        + CrateHandleInterface<P>
        + IsPrivate<Error = CrateError>
        + Send
        + Sync
        + 'static,
{
    async fn try_publish_crate_tree(
        &self,
        root_crate_name: &str,
        dry_run: bool,
    ) -> Result<(), WorkspaceError> {
        info!("Building dependency graph …");
        let dependency_graph = self.generate_dependency_tree().await?;

        /* --------------------------------------------------------- *
         * Step 1 – locate the root node and all transitively‑reached
         *          dependency nodes (outgoing edges).
         * --------------------------------------------------------- */
        let mut name_to_index = HashMap::<String, NodeIndex>::new();
        for idx in dependency_graph.node_indices() {
            let name = dependency_graph
                .node_weight(idx)
                .expect("node without weight")
                .clone();
            name_to_index.insert(name, idx);
        }

        let root_index = name_to_index
            .get(root_crate_name)
            .ok_or_else(|| WorkspaceError::CrateError(CrateError::CrateNotFoundInWorkspace {
                crate_name: root_crate_name.to_string(),
            }))?;

        let mut reachable = HashSet::new();
        let mut bfs = Bfs::new(&dependency_graph, *root_index);
        while let Some(idx) = bfs.next(&dependency_graph) {
            let name = dependency_graph
                .node_weight(idx)
                .expect("node without weight")
                .clone();
            reachable.insert(name);
        }
        debug!(
            "Crate‑tree rooted at '{}' contains {} crates",
            root_crate_name,
            reachable.len()
        );

        /* --------------------------------------------------------- *
         * Step 2 – map crate‑name → CrateHandle for the selected set
         * --------------------------------------------------------- */
        let mut name_to_handle =
            BTreeMap::<String, Arc<tokio::sync::Mutex<H>>>::new();
        for crate_handle in self.crates().iter() {
            let guard = crate_handle.lock().await;
            let name = guard.name().to_string();
            if reachable.contains(&name) {
                name_to_handle.insert(name, crate_handle.clone());
            }
        }

        if !name_to_handle.contains_key(root_crate_name) {
            warn!(
                "Root crate '{}' is not part of this workspace; nothing to publish.",
                root_crate_name
            );
            return Ok(());
        }

        /* --------------------------------------------------------- *
         * Step 3 – publish in reverse topological order (to match the
         *          original implementation semantics).
         * --------------------------------------------------------- */
        let topo_order = toposort(&dependency_graph, None).map_err(|cycle| {
            WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                cycle_node_id: cycle.node_id(),
            }
        })?;

        for node_idx in topo_order.iter().rev() {
            let crate_name = dependency_graph
                .node_weight(*node_idx)
                .expect("node without weight");

            if !reachable.contains(crate_name) {
                continue; // outside requested tree
            }

            let Some(crate_handle) = name_to_handle.get(crate_name) else { continue };

            let mut guard = crate_handle.lock().await;

            if guard.is_private().await? {
                debug!("SKIP: crate '{}' is marked private.", crate_name);
                continue;
            }

            let version = guard.version()?;

            info!("------------------------------------------------");
            info!("Crate:   {crate_name}");
            info!("Version: {version}");

            if is_crate_version_published_on_crates_io(crate_name, &version).await? {
                info!("SKIP: {crate_name}@{version} already on crates.io");
                continue;
            }

            info!("Attempting to publish {crate_name}@{version} …");
            match guard.try_publish(dry_run).await {
                Ok(_) => { /* success */ }
                Err(e) => {
                    error!(
                        "FATAL: could not publish {crate_name}@{version}: {:?}",
                        e
                    );
                    return Err(e.into());
                }
            }
        }

        info!("Done – all crates in the '{}' tree are published (or skipped).", root_crate_name);
        Ok(())
    }
}
