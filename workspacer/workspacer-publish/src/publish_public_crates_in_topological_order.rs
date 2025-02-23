// ---------------- [ File: src/publish_public_crates_in_topological_order.rs ]
crate::ix!();

#[async_trait]
impl<P, H: TryPublish<Error=CrateError> + CrateHandleInterface<P>> TryPublish for Workspace<P, H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    type Error = WorkspaceError;

    /// Publishes all public crates in a workspace in topological order,
    /// skipping those that are already on crates.io or that produce an
    /// "already exists" error during publish.
    async fn try_publish(&self, dry_run: bool) -> Result<(), Self::Error> {
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

            let cargo_toml = crate_handle.cargo_toml();

            let package_section = cargo_toml
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
                    match crate_handle.try_publish(dry_run).await {
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

#[cfg(test)]
#[disable]
mod test_try_publish {
    use super::*;
    use workspacer_3p::tokio;
    use std::collections::{BTreeMap, VecDeque};
    use petgraph::prelude::*;
    use crate::{ 
        // your types:
        TryPublish, 
        // We'll define or import:
        is_crate_version_published_on_crates_io, 
        // plus any others you need.
    };

    // ------------------------------------------------------------------------
    // 1) Mocking the workspace
    // ------------------------------------------------------------------------

    /// A minimal "mock" struct simulating a workspace with:
    /// - a precomputed dependency graph scenario,
    /// - a list of mock crates that define name, version, is_private, already_published,
    ///   and success/failure for `try_publish`.
    #[derive(Debug)]
    struct MockWorkspace {
        /// If `Some(...)`, we have a valid graph. If `None`, we force a cycle error scenario.
        pub maybe_graph: Option<DiGraph<String, ()>>,
        /// The topological order of crate names if no cycle
        pub topo_order: Option<Vec<String>>,
        /// Mapping crate_name -> MockCrateHandle
        pub crates: BTreeMap<String, MockCrateHandle>,
    }

    // We'll define a small struct for each crate's scenario:
    #[derive(Debug, Clone)]
    struct MockCrateHandle {
        pub is_private: bool,
        pub crate_name: String,
        pub crate_version: semver::Version,
        /// True => we say "already on crates.io"
        pub already_published: bool,
        /// If None => success. If Some(msg) => error. If that msg = "already exists" => treat as skip
        pub publish_error: Option<String>,
    }

    // We define how to get cargo_toml name, version, is_private, 
    // and try_publish(dry_run). We'll rely on these as "the crate handle" in the real code.

    impl MockCrateHandle {
        pub fn new_public(
            name: &str, 
            version: &str
        ) -> Self {
            Self {
                is_private: false,
                crate_name: name.to_string(),
                crate_version: semver::Version::parse(version).unwrap(),
                already_published: false,
                publish_error: None,
            }
        }

        pub fn new_private(name: &str) -> Self {
            Self {
                is_private: true,
                crate_name: name.to_string(),
                crate_version: semver::Version::new(0,1,0),
                already_published: false,
                publish_error: None,
            }
        }
    }

    // We'll define the trait calls that your real code uses (like is_private, name, version, try_publish).

    impl MockCrateHandle {
        pub fn is_private(&self) -> Result<bool, CrateError> {
            Ok(self.is_private)
        }
        pub fn name(&self) -> &str {
            &self.crate_name
        }
        pub fn version(&self) -> Result<semver::Version, CrateError> {
            Ok(self.crate_version.clone())
        }
        pub async fn try_publish(&self, dry_run: bool) -> Result<(), CrateError> {
            if dry_run {
                // log skip
                info!("DRY RUN skip publishing {}@{}", self.crate_name, self.crate_version);
                return Ok(());
            }
            if let Some(ref msg) = self.publish_error {
                if msg.contains("already exists") {
                    warn!("SKIP: cargo says {}@{} already exists => treat as success", self.crate_name, self.crate_version);
                    Ok(())
                } else {
                    error!("Publish error for {}@{}: {}", self.crate_name, self.crate_version, msg);
                    Err(CrateError::CargoPublishFailedForCrateWithExitCode {
                        crate_name: self.crate_name.clone(),
                        crate_version: self.crate_version.clone(),
                        exit_code: Some(1),
                    })
                }
            } else {
                Ok(())
            }
        }
    }

    // We'll also define a function to mimic is_crate_version_published_on_crates_io
    // returning `true` if self.already_published is set.

    async fn mock_is_crate_version_published_on_crates_io(
        crate_name: &str, 
        crate_version: &semver::Version,
        crates: &BTreeMap<String, MockCrateHandle>
    ) -> Result<bool, WorkspaceError> {
        // Just check the handle's already_published
        if let Some(ch) = crates.get(crate_name) {
            Ok(ch.already_published)
        } else {
            // can't find => error or false
            Ok(false)
        }
    }

    // We define a method to mock generate_dependency_tree. If maybe_graph is None => we produce a cycle error.

    impl MockWorkspace {
        pub async fn generate_dependency_tree(&self) -> Result<DiGraph<String, ()>, WorkspaceError> {
            if let Some(ref graph) = self.maybe_graph {
                Ok(graph.clone())
            } else {
                // means we simulate a cycle error
                Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph {
                    cycle_node_id: NodeIndex::new(999),
                })
            }
        }

        // We'll also define a function to get the topological order from `topo_order`,
        // or do a real petgraph toposort if we prefer. We'll just use your code's approach:
        pub fn toposort_mock(&self) -> Result<Vec<String>, WorkspaceError> {
            if let Some(ref order) = self.topo_order {
                Ok(order.clone())
            } else {
                // fallback => real approach
                Err(WorkspaceError::InvalidWorkspace { invalid_workspace_path: "mock path".into() })
            }
        }
    }

    // We'll define an extension trait for the test that replicates `try_publish(dry_run)` logic, but referencing the mocks
    #[async_trait]
    trait MockTryPublish {
        async fn try_publish_workspace(&self, dry_run: bool) -> Result<(), WorkspaceError>;
    }

    #[async_trait]
    impl MockTryPublish for MockWorkspace {
        async fn try_publish_workspace(&self, dry_run: bool) -> Result<(), WorkspaceError> {
            // 1) generate dep graph
            let _graph = self.generate_dependency_tree().await?;

            // 2) topological order
            let topo = self.toposort_mock()?;

            // 3) build crate_name -> handle map
            // It's basically self.crates

            for crate_node_name in topo {
                let handle = self.crates.get(&crate_node_name)
                    .expect("We expect handle to be in the map");
                let is_private = handle.is_private()?;
                if is_private {
                    log::debug!("SKIP private crate {crate_node_name}");
                    continue;
                }
                let crate_name = handle.name();
                let crate_version = handle.version().expect("should have version");
                // check if published
                if mock_is_crate_version_published_on_crates_io(crate_name, &crate_version, &self.crates).await? {
                    log::info!("SKIP: {crate_name}@{crate_version} is already on crates.io");
                } else {
                    log::info!("Attempting to publish {crate_name}@{crate_version}...");
                    match handle.try_publish(dry_run).await {
                        Ok(_) => { /* success or 'already exists' skip */}
                        Err(e) => {
                            log::error!("FATAL: Could not publish {crate_name}@{crate_version}");
                            return Err(e.into()); // map into WorkspaceError
                        }
                    }
                }
            }

            Ok(())
        }
    }

    // ------------------------------------------------------------------------
    // 2) Tests
    // ------------------------------------------------------------------------

    use crate::{ CrateError, WorkspaceError }; // or your actual paths

    #[tokio::test]
    async fn test_cycle_in_graph() {
        let ws = MockWorkspace {
            maybe_graph: None, // => force cycle error
            topo_order: None,  // won't matter
            crates: BTreeMap::new(),
        };
        let result = ws.try_publish_workspace(false).await;
        match result {
            Err(WorkspaceError::CycleDetectedInWorkspaceDependencyGraph { cycle_node_id }) => {
                assert_eq!(cycle_node_id.index(), 999);
            }
            other => panic!("Expected cycle error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_skip_private_crate() {
        // We'll define a simple graph with 1 node: "private_crate"
        let mut graph = DiGraph::new();
        let node_idx = graph.add_node("private_crate".to_string());
        // no edges
        let topo_order = vec!["private_crate".to_string()];

        // define a private crate handle
        let mut crates_map = BTreeMap::new();
        crates_map.insert("private_crate".to_string(), MockCrateHandle {
            is_private: true,
            crate_name: "private_crate".to_string(),
            crate_version: semver::Version::new(0,1,0),
            already_published: false,
            publish_error: None,
        });

        let ws = MockWorkspace {
            maybe_graph: Some(graph),
            topo_order: Some(topo_order),
            crates: crates_map,
        };

        let result = ws.try_publish_workspace(false).await;
        assert!(result.is_ok(), "We skip private crate => success overall");
    }

    #[tokio::test]
    async fn test_skip_already_published_crate() {
        let mut graph = DiGraph::new();
        graph.add_node("crate_a".to_string());
        let topo_order = vec!["crate_a".to_string()];
        let mut crates_map = BTreeMap::new();
        crates_map.insert("crate_a".to_string(), MockCrateHandle {
            is_private: false,
            crate_name: "crate_a".to_string(),
            crate_version: semver::Version::new(1,2,3),
            already_published: true, // => skip
            publish_error: None,
        });

        let ws = MockWorkspace {
            maybe_graph: Some(graph),
            topo_order: Some(topo_order),
            crates: crates_map,
        };

        let result = ws.try_publish_workspace(false).await;
        assert!(result.is_ok(), "Already published => skip => no error");
    }

    #[tokio::test]
    async fn test_publish_succeeds() {
        // We have a single crate, not published, not private, no publish error
        let mut graph = DiGraph::new();
        graph.add_node("crate_a".to_string());
        let topo = vec!["crate_a".to_string()];

        let handle = MockCrateHandle::new_public("crate_a","0.1.0");
        let mut map = BTreeMap::new();
        map.insert("crate_a".to_string(), handle);

        let ws = MockWorkspace {
            maybe_graph: Some(graph),
            topo_order: Some(topo),
            crates: map,
        };

        let result = ws.try_publish_workspace(false).await;
        assert!(result.is_ok(), "publish should succeed with no errors");
    }

    #[tokio::test]
    async fn test_publish_already_exists_treated_as_success() {
        // We'll define a crate that is not published, but `try_publish` will produce "already exists" error => skip
        let mut graph = DiGraph::new();
        graph.add_node("crate_a".to_string());
        let topo = vec!["crate_a".to_string()];

        let mut handle = MockCrateHandle::new_public("crate_a","0.1.0");
        handle.already_published = false;
        handle.publish_error = Some("already exists".to_string());
        let mut map = BTreeMap::new();
        map.insert("crate_a".to_string(), handle);

        let ws = MockWorkspace {
            maybe_graph: Some(graph),
            topo_order: Some(topo),
            crates: map,
        };

        let result = ws.try_publish_workspace(false).await;
        assert!(result.is_ok(), "already exists => skip => success");
    }

    #[tokio::test]
    async fn test_publish_fails_other_error() {
        let mut graph = DiGraph::new();
        graph.add_node("crate_a".to_string());
        let topo = vec!["crate_a".to_string()];

        let mut handle = MockCrateHandle::new_public("crate_a","1.0.0");
        handle.publish_error = Some("something else".to_string()); // => real error
        let mut map = BTreeMap::new();
        map.insert("crate_a".to_string(), handle);

        let ws = MockWorkspace {
            maybe_graph: Some(graph),
            topo_order: Some(topo),
            crates: map,
        };

        let result = ws.try_publish_workspace(false).await;
        match result {
            Err(WorkspaceError::CrateError(_)) => {
                // Or potentially a different variant, depending on your `.into()` logic 
                // which maps CrateError -> WorkspaceError. 
            }
            Ok(_) => panic!("Expected failure for a real error message"),
            other => panic!("Expected CrateError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_dry_run_skips_actual_publish() {
        let mut graph = DiGraph::new();
        graph.add_node("crate_a".to_string());
        let topo = vec!["crate_a".to_string()];

        let mut handle = MockCrateHandle::new_public("crate_a","1.2.3");
        // If it tries to publish for real, it would fail, but in a dry_run => skip
        handle.publish_error = Some("would fail if real".to_string());
        let mut map = BTreeMap::new();
        map.insert("crate_a".to_string(), handle);

        let ws = MockWorkspace {
            maybe_graph: Some(graph),
            topo_order: Some(topo),
            crates: map,
        };

        let result = ws.try_publish_workspace(true).await;
        assert!(result.is_ok(), "dry_run => skip => success");
    }
}
