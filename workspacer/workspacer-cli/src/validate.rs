crate::ix!();

/// Validate everything (or just one crate)
#[derive(Debug, StructOpt)]
pub enum ValidateSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        workspace_path: PathBuf,
    },
}

impl ValidateSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
