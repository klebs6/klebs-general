crate::ix!();

/// Generate coverage reports
#[derive(Debug, StructOpt)]
pub enum CoverageSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl CoverageSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}

