crate::ix!();

/// Recursively build child nodes up to `max_levels`.
pub fn build_children_rec(
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
