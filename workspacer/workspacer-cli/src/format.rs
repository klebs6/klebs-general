crate::ix!();

/// Format imports in all crates or a single crate
#[derive(Debug, StructOpt)]
pub enum FormatSubcommand {
    Imports {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    AllImports {
        #[structopt(long = "path")]
        workspace_path: PathBuf,
    },
}

impl FormatSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
