crate::ix!();

/// Register crate files in the internal database
#[derive(Debug, StructOpt)]
pub enum RegisterSubcommand {
    CrateFiles {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    AllFiles {
        #[structopt(long = "path")]
        workspace_path: PathBuf,
    },
}

impl RegisterSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
