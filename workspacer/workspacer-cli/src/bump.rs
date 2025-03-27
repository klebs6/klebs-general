crate::ix!();

/// Bump the version(s) (workspace or single crate or just one crate by name)
#[derive(Debug, StructOpt)]
pub enum BumpSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_path: PathBuf,
    },
    Workspace {
        path: PathBuf,
    },
}

impl BumpSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
