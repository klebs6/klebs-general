// ---------------- [ File: workspacer-cli/src/tree.rs ]
crate::ix!();

#[derive(Debug, structopt::StructOpt)]
pub struct TreeSubcommand {
    #[structopt(long, default_value = "999")]
    levels: usize,

    /// Whether to show crate versions in the tree
    #[structopt(long)]
    show_version: bool,

    /// Whether to show crate path in the tree
    #[structopt(long)]
    show_path: bool,

    /// Possibly a verbose flag
    #[structopt(long)]
    verbose: bool,
}

impl TreeSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // 1) Build or open your workspace
        let path = std::env::current_dir()?;
        let ws = match Workspace::<PathBuf, CrateHandle>::new(&path).await {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Error building workspace: {:?}", e);
                return Err(e);
            }
        };

        // 2) Call `build_workspace_tree`
        let tree = ws.build_workspace_tree(self.levels, self.verbose).await?;

        // 3) Print it
        // We can do:  let output_str = tree.render(self.show_version, self.show_path);
        let output_str = tree.render(self.show_version, self.show_path);
        println!("{}", output_str);

        Ok(())
    }
}
