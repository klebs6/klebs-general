// ---------------- [ File: workspacer-cli/src/tree.rs ]
crate::ix!();

#[derive(Debug, StructOpt, Getters, Setters, Builder)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct TreeSubcommand {
    /// How many dependency levels to recurse (default 999 means “no real limit”).
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

    /// If provided, the name of a crate to act as our root (entrypoint).
    /// Otherwise, we show all top-level roots in the workspace.
    #[structopt(long)]
    crate_name: Option<String>,
}

impl TreeSubcommand {
    /// Executes the “tree” subcommand, building either the full workspace tree
    /// or a focused subtree if `crate_name` is specified.
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        use tracing::{debug, error, info, trace};

        trace!(
            "Entering TreeSubcommand::run (levels={}, show_version={}, show_path={}, verbose={}, crate_name={:?})",
            self.levels(),
            self.show_version(),
            self.show_path(),
            self.verbose(),
            self.crate_name(),
        );

        let path = match std::env::current_dir() {
            Ok(p) => {
                debug!("Current dir = {:?}", p);
                p
            }
            Err(e) => {
                error!("Failed to get current dir: {:?}", e);
                return Err(WorkspaceError::IoError {
                    io_error: Arc::new(e),
                    context: "Could not obtain current working directory".to_string(),
                });
            }
        };

        let ws = match Workspace::<PathBuf, CrateHandle>::new(&path).await {
            Ok(w) => {
                info!("Successfully built workspace at path: {:?}", path);
                w
            }
            Err(e) => {
                error!("Error building workspace: {:?}", e);
                return Err(e);
            }
        };

        // Decide whether we want the full workspace tree or just a single crate subtree
        let tree = if let Some(name) = self.crate_name() {
            info!("Building subtree for specified crate: {}", name);
            ws.build_workspace_subtree(name, *self.levels(), *self.verbose())
                .await?
        } else {
            debug!("No specific crate requested => building full workspace tree");
            ws.build_workspace_tree(*self.levels(), *self.verbose()).await?
        };

        let output_str = tree.render(*self.show_version(), *self.show_path());
        println!("{}", output_str);

        Ok(())
    }
}
