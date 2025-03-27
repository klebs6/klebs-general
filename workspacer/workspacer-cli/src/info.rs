crate::ix!();

/// Print general info about the workspace or crate
#[derive(Debug, StructOpt)]
pub enum InfoSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl InfoSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
