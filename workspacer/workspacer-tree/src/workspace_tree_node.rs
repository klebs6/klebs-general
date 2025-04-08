crate::ix!();

#[derive(Debug)]
pub struct WorkspaceTreeNode {
    crate_name:   String,
    crate_version: Option<SemverVersion>,
    crate_path:   PathBuf,
    children:     Vec<WorkspaceTreeNode>,
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
