crate::ix!();

/// Document the workspace or crate (like cargo doc)
#[derive(Debug, StructOpt)]
pub enum DocumentSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl DocumentSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
