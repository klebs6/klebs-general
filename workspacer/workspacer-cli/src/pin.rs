// ---------------- [ File: workspacer-cli/src/pin.rs ]
crate::ix!();

/// Pin wildcard dependencies in workspace or crate
#[derive(Debug, StructOpt)]
pub enum PinSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl PinSubcommand {
    pub async fn run(&self) -> Result<(),WorkspaceError> {
        todo!();
    }
}
