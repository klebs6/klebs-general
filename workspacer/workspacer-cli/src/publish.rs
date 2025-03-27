crate::ix!();

/// Publish workspace or crate
#[derive(Debug, StructOpt)]
pub enum PublishSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl PublishSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
