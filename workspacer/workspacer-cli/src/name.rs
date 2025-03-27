crate::ix!();

#[derive(Debug, StructOpt)]
pub enum NameSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl NameSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
