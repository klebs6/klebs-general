// ---------------- [ File: workspacer-cli/src/watch.rs ]
crate::ix!();

/// Watch for changes
#[derive(Debug, StructOpt)]
pub enum WatchSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        workspace_path: PathBuf,
    },
}

impl WatchSubcommand {
    pub async fn run(&self) -> Result<(),WorkspaceError> {
        todo!();
    }
}
