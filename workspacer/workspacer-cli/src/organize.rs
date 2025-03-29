// ---------------- [ File: workspacer-cli/src/organize.rs ]
crate::ix!();

/// Organize the workspace or a single crate
#[derive(Debug, StructOpt)]
pub enum OrganizeSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl OrganizeSubcommand {
    pub async fn run(&self) -> Result<(),WorkspaceError> {
        todo!();
    }
}
