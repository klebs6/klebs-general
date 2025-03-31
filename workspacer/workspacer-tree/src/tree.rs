// ---------------- [ File: workspacer-tree/src/tree.rs ]
crate::ix!();

#[derive(Debug)]
pub struct WorkspaceDependencyTree {
    // The top-level crates with no incoming edges (roots).
    // Not public; we’ll provide only internal constructor & printing.
    roots: Vec<WorkspaceTreeNode>,
}

#[derive(Debug)]
pub struct WorkspaceTreeNode {
    crate_name:   String,
    crate_version: Option<SemverVersion>,
    crate_path:   PathBuf,
    children:     Vec<WorkspaceTreeNode>,
}

impl WorkspaceDependencyTree {
    /// Internal constructor for the final tree.
    pub fn new(roots: Vec<WorkspaceTreeNode>) -> Self {
        Self { roots }
    }

    /// Helper for checking whether we’re empty (no crates).
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// Render the entire forest as a string.  
    /// `show_version`, `show_path` => whether to display version/path in output.
    pub fn render(
        &self,
        show_version: bool,
        show_path: bool,
    ) -> String {
        if self.is_empty() {
            return "(no crates in workspace)".to_string();
        }

        let mut out = String::new();
        for (i, root) in self.roots.iter().enumerate() {
            // Indentation starts at level=0:
            root.render_recursive(0, show_version, show_path, &mut out);
            if i + 1 < self.roots.len() {
                out.push('\n'); // blank line between top-level roots
            }
        }
        out
    }
}

impl WorkspaceTreeNode {
    /// Internal constructor for an individual node.
    pub fn new(
        crate_name: String,
        crate_version: Option<SemverVersion>,
        crate_path: PathBuf,
    ) -> Self {
        Self {
            crate_name,
            crate_version,
            crate_path,
            children: Vec::new(),
        }
    }

    /// Recursively push child nodes
    pub fn add_child(&mut self, child: WorkspaceTreeNode) {
        self.children.push(child);
    }

    /// Recursively renders `self` into the output string with indentation.
    /// `level` controls how many spaces to indent. 
    pub fn render_recursive(
        &self,
        level: usize,
        show_version: bool,
        show_path: bool,
        out: &mut String,
    ) {
        // Indent with 2 spaces * level
        let indent = "  ".repeat(level);

        // e.g. "my_crate (v1.2.3)  [/some/path]"
        let mut line = format!("{}{}", indent, self.crate_name);

        if show_version {
            if let Some(ref v) = self.crate_version {
                line.push_str(&format!(" (v{})", v));
            }
        }

        if show_path {
            line.push_str(&format!("  [{}]", self.crate_path.display()));
        }

        out.push_str(&line);
        out.push('\n');

        // Recurse on children
        for child in &self.children {
            child.render_recursive(level + 1, show_version, show_path, out);
        }
    }
}

#[async_trait]
pub trait WorkspaceTreeBuilder {
    /// Build the workspace dependency tree up to `levels` levels deep,
    /// optionally toggling “verbose,” or ignoring it as desired.
    async fn build_workspace_tree(
        &self,
        levels: usize,
        verbose: bool,
    ) -> Result<WorkspaceDependencyTree, WorkspaceError>;
}

// We need to ensure P, H meet the strict bounds that your Workspace<P,H> requires:
//   P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static
//   H: CrateHandleInterface<P> + Send + Sync + 'static
#[async_trait]
impl<P,H> WorkspaceTreeBuilder for Workspace<P,H>
where
    P: From<PathBuf> + AsRef<Path> + Send + Sync + 'static,
    H: CrateHandleInterface<P> + Send + Sync + 'static,
{
    async fn build_workspace_tree(
        &self,
        levels: usize,
        verbose: bool,
    ) -> Result<WorkspaceDependencyTree, WorkspaceError>
    {
        // 1) We gather all crates in the workspace:
        let all_crates = self.crates(); 
        //  ^ typically returns e.g. &Vec<Arc<AsyncMutex<H>>>, or an iterator

        // Build a petgraph with (node=crate name), (edge=dependency).
        let mut graph = Graph::<String, ()>::new();
        let mut name_to_idx = BTreeMap::new();

        // For storing info about each crate => version + path + internal deps
        // We'll store Option<SemverVersion> for the crate_version, plus the PathBuf
        let mut crate_info: Vec<(String, Option<SemverVersion>, PathBuf, Vec<String>)> = Vec::new();

        // Lock each crate, gather metadata
        for arc_h in all_crates {
            let guard = arc_h.lock().await;

            let crate_name = guard.name().to_string();

            // Suppose `guard.version()?` yields an Option<SemverVersion> or a SemverVersion?
            // If it yields `SemverVersion`, wrap it in Some(...). If it can be None, store that.
            let crate_version = Some(guard.version()?); 

            // Suppose guard.as_ref() yields a P that is basically a Path or something?
            // We'll get a PathBuf from it:
            let crate_path: PathBuf = guard.as_ref().to_path_buf();

            // We also gather “internal_deps,” i.e. other crates in the workspace.
            let internal_deps: Vec<String> = guard.internal_dependencies().await?;

            // Add node to the graph
            let node_idx = graph.add_node(crate_name.clone());
            name_to_idx.insert(crate_name.clone(), node_idx);

            // Put in crate_info for later lookup
            crate_info.push((crate_name, crate_version, crate_path, internal_deps));
        }

        // 2) Add edges
        for (crate_name, _v, _path, deps) in &crate_info {
            let src_idx = name_to_idx[crate_name];
            for dep_name in deps {
                if let Some(&dst_idx) = name_to_idx.get(dep_name) {
                    graph.add_edge(src_idx, dst_idx, ());
                }
            }
        }

        // 3) Identify “roots” => nodes with no incoming edges
        let mut indeg = BTreeMap::new();
        for n in graph.node_indices() {
            indeg.insert(n, 0usize);
        }
        for e in graph.edge_indices() {
            let (s, d) = graph.edge_endpoints(e).unwrap();
            // increment the in-degree of d
            *indeg.get_mut(&d).unwrap() += 1;
        }

        let roots: Vec<_> = indeg
            .iter()
            .filter_map(|(node_idx, count)| {
                if *count == 0 {
                    Some(*node_idx)
                } else {
                    None
                }
            })
            .collect();

        // 4) Build the “WorkspaceDependencyTree”
        let mut out_roots = Vec::new();

        for root_idx in roots {
            let crate_name = graph[root_idx].clone(); // The node’s name
            let (version, path) = crate_info_for_name(&crate_info, &crate_name);
            
            let mut root_node = WorkspaceTreeNode::new(
                crate_name,
                version,
                path,
            );
            
            // Recursively gather children (if levels>0)
            if levels > 0 {
                build_children_rec(
                    &graph,
                    &crate_info,
                    root_idx,
                    1,
                    levels,
                    &mut root_node
                );
            }

            out_roots.push(root_node);
        }

        let tree = WorkspaceDependencyTree::new(out_roots);
        Ok(tree)
    }
}

/// Utility: find the version/path for the given crate_name in our crate_info list.
fn crate_info_for_name(
    crate_info: &[(String, Option<SemverVersion>, PathBuf, Vec<String>)],
    name: &str,
) -> (Option<SemverVersion>, PathBuf) 
{
    for (cn, cv, p, _) in crate_info {
        if cn == name {
            return (cv.clone(), p.clone());
        }
    }
    // fallback if not found
    (None, PathBuf::new())
}

/// Recursively build child nodes up to `max_levels`.
fn build_children_rec(
    graph: &Graph<String, ()>,
    crate_info: &[(String, Option<SemverVersion>, PathBuf, Vec<String>)],
    node_idx: petgraph::graph::NodeIndex,
    current_level: usize,
    max_levels: usize,
    parent_node: &mut WorkspaceTreeNode,
) {
    if current_level >= max_levels {
        return; // do not go deeper
    }

    // For each outgoing edge => build child
    for edge in graph.edges_directed(node_idx, petgraph::Direction::Outgoing) {
        let child_idx = edge.target();
        let crate_name = &graph[child_idx];

        let (version, path) = crate_info_for_name(crate_info, crate_name);

        let mut child_node = WorkspaceTreeNode::new(
            crate_name.to_string(),
            version,
            path,
        );

        // Recurse
        build_children_rec(
            graph, crate_info,
            child_idx,
            current_level + 1,
            max_levels,
            &mut child_node,
        );

        parent_node.add_child(child_node);
    }
}
