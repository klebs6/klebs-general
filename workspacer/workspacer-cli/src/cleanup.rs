crate::ix!();

/// Cleanup the workspace or a single crate
#[derive(Debug, StructOpt)]
pub enum CleanupSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl CleanupSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
