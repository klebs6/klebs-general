crate::ix!();

#[async_trait]
pub trait BuildWorkspaceSubTree {
    async fn build_workspace_subtree(
        &self,
        crate_name: &str,
        levels: usize,
        verbose: bool,
    ) -> Result<WorkspaceDependencyTree, WorkspaceError>;
}

#[async_trait]
impl<P, H> BuildWorkspaceSubTree for Workspace<P, H>
where
    P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'static,
    H: CrateHandleInterface<P> + Send + Sync + 'static,
{
    /// Builds a tree focusing on a single crate as the “root” (entrypoint),
    /// recursing down its dependencies up to `levels`.
    async fn build_workspace_subtree(
        &self,
        crate_name: &str,
        levels: usize,
        verbose: bool,
    ) -> Result<WorkspaceDependencyTree, WorkspaceError> {
        use petgraph::visit::EdgeRef;
        use petgraph::{graph::Graph, Direction};
        use std::collections::BTreeMap;
        use tracing::{debug, error, info, trace};

        trace!(
            "Entering build_workspace_subtree(crate_name={:?}, levels={:?}, verbose={:?})",
            crate_name,
            levels,
            verbose
        );

        // 1) Gather all crates in the workspace
        let all_crates = self.crates();
        debug!("Number of crates in workspace: {}", all_crates.len());

        // Build a petgraph with (node=crate name), (edge=dependency).
        let mut graph = Graph::<String, ()>::new();
        let mut name_to_idx = BTreeMap::new();

        // For storing info about each crate => version + path + internal deps
        let mut crate_info: Vec<(String, Option<SemverVersion>, PathBuf, Vec<String>)> = Vec::new();

        // Lock each crate handle, gather metadata
        for arc_h in all_crates {
            let guard = arc_h.lock().await;
            let nm = guard.name().to_string();
            trace!("Collecting metadata for crate: {}", nm);

            let ver = match guard.version() {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("Error retrieving version for crate `{}`: {:?}", nm, e);
                    None
                }
            };

            let crate_path: PathBuf = guard.as_ref().to_path_buf();
            let internal_deps = guard.internal_dependencies().await.unwrap_or_else(|e| {
                error!(
                    "Failed to retrieve internal_dependencies for crate `{}`: {:?}",
                    nm, e
                );
                vec![]
            });

            let node_idx = graph.add_node(nm.clone());
            name_to_idx.insert(nm.clone(), node_idx);

            crate_info.push((nm, ver, crate_path, internal_deps));
        }

        // 2) Add edges for each “internal dependency”
        for (src_name, _ver, _path, deps) in &crate_info {
            let src_idx = name_to_idx[src_name];
            for dep_name in deps {
                if let Some(&dst_idx) = name_to_idx.get(dep_name) {
                    graph.add_edge(src_idx, dst_idx, ());
                } else {
                    debug!("Dependency `{}` not found in name_to_idx, ignoring", dep_name);
                }
            }
        }

        // 3) Find the node index for `crate_name`
        let root_idx = match name_to_idx.get(crate_name) {
            Some(idx) => *idx,
            None => {
                error!(
                    "Requested root crate `{}` not found in this workspace",
                    crate_name
                );
                return Err(WorkspaceError::InvalidWorkspace {
                    invalid_workspace_path: self.as_ref().to_path_buf(),
                });
            }
        };

        // 4) Build up a single root node for that crate
        let (root_ver, root_path) = crate_info_for_name(&crate_info, crate_name);
        let mut root_node = WorkspaceTreeNode::new(crate_name.to_string(), root_ver, root_path);

        // 5) Recurse down its dependencies if levels > 0
        if levels > 0 {
            build_children_rec(
                &graph,
                &crate_info,
                root_idx,
                1,
                levels,
                &mut root_node,
            );
        }

        // 6) Return a tree with just one root
        let tree = WorkspaceDependencyTree::new(vec![root_node]);
        info!(
            "build_workspace_subtree => returning tree with 1 root: `{}`",
            crate_name
        );
        Ok(tree)
    }
}

#[cfg(test)]
mod test_build_workspace_subtree {
    use super::*;

    /// Helper that creates a mock workspace directory with two crates:
    /// crateA depends on crateB.
    /// Then returns the path so we can build the workspace from it.
    async fn create_mock_workspace_with_deps() -> PathBuf {
        let tmp = tempdir().expect("Failed to create temp dir");
        let root_path = tmp.path().to_path_buf();

        // Write top-level Cargo.toml for the workspace
        let cargo_toml_content = r#"
            [workspace]
            members = ["crateA","crateB"]
        "#;
        {
            let mut f = std::fs::File::create(root_path.join("Cargo.toml"))
                .expect("Failed to create workspace Cargo.toml");
            f.write_all(cargo_toml_content.as_bytes())
                .expect("Failed to write workspace Cargo.toml");
        }

        // 1) crateB => no internal dependencies
        let crate_b = root_path.join("crateB");
        std::fs::create_dir(&crate_b).expect("Failed to create crateB dir");
        {
            let mut f = std::fs::File::create(crate_b.join("Cargo.toml"))
                .expect("Failed to create crateB Cargo.toml");
            let contents = r#"
                [package]
                name = "crateB"
                version = "0.1.0"

                [dependencies]
                # no internal dependencies
            "#;
            f.write_all(contents.as_bytes())
                .expect("Failed to write crateB Cargo.toml");
        }

        // 2) crateA => depends on crateB (path-based)
        let crate_a = root_path.join("crateA");
        std::fs::create_dir(&crate_a).expect("Failed to create crateA dir");
        {
            let mut f = std::fs::File::create(crate_a.join("Cargo.toml"))
                .expect("Failed to create crateA Cargo.toml");
            let contents = r#"
                [package]
                name = "crateA"
                version = "0.2.0"

                [dependencies]
                crateB = { path = "../crateB" }
            "#;
            f.write_all(contents.as_bytes())
                .expect("Failed to write crateA Cargo.toml");
        }

        root_path
    }

    #[traced_test]
    async fn test_build_workspace_subtree_crateA() {
        let ws_path = create_mock_workspace_with_deps().await;
        let workspace = Workspace::<PathBuf, CrateHandle>::new(&ws_path)
            .await
            .expect("Should build workspace from mock directory");

        // crateA depends on crateB, so subtree for crateA should contain both A and B
        let tree_a = workspace
            .build_workspace_subtree("crateA", 5, false)
            .await
            .expect("Should build subtree for crateA");

        let rendered_a = tree_a.render(true, false);
        info!("Subtree for crateA:\n{}", rendered_a);
        assert!(
            rendered_a.contains("crateA"),
            "Should contain crateA in subtree"
        );
        assert!(
            rendered_a.contains("crateB"),
            "Should contain crateB in crateA's subtree"
        );

        // crateB => no dependencies
        let tree_b = workspace
            .build_workspace_subtree("crateB", 5, false)
            .await
            .expect("Should build subtree for crateB");

        let rendered_b = tree_b.render(true, false);
        trace!("Subtree for crateB:\n{}", rendered_b);
        assert!(
            rendered_b.contains("crateB"),
            "Should contain crateB in subtree"
        );
        assert!(
            !rendered_b.contains("crateA"),
            "crateB has no parent => should not contain crateA"
        );
    }
}
