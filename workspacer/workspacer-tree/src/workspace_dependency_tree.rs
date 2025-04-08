crate::ix!();

#[derive(Debug)]
pub struct WorkspaceDependencyTree {
    // The top-level crates with no incoming edges (roots).
    // Not public; we’ll provide only internal constructor & printing.
    roots: Vec<WorkspaceTreeNode>,
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
