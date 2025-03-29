// ---------------- [ File: workspacer-cli/src/meta.rs ]
crate::ix!();

/// Show cargo metadata
#[derive(Debug, StructOpt)]
pub enum MetaSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl MetaSubcommand {
    pub async fn run(&self) -> Result<(),WorkspaceError> {
        todo!();
    }
}
