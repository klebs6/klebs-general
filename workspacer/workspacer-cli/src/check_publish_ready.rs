crate::ix!();

/// Check if this crate or entire workspace is ready for publishing
#[derive(Debug, StructOpt)]
pub enum CheckPublishReadySubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl CheckPublishReadySubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
