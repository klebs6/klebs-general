crate::ix!();

/// Describe the workspace or crate
#[derive(Debug, StructOpt)]
pub enum DescribeSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl DescribeSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
