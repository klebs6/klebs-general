crate::ix!();

/// Write or update README files
#[derive(Debug, StructOpt)]
pub enum WriteSubcommand {
    CrateReadme {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    AllReadmes {
        #[structopt(long = "workspace")]
        workspace_path: PathBuf,
    },
}

impl WriteSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
