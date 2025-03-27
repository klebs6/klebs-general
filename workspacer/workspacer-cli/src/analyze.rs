crate::ix!();

/// Analyze the current workspace or crate (or a specified crate by name)
#[derive(Debug, StructOpt)]
pub enum AnalyzeSubcommand {
    /// If specified, analyzes only that crate (must be run from a workspace)
    Crate {
        #[structopt(long = "crate")]
        crate_path: PathBuf,
    },
    Workspace {
        path: PathBuf,
    },
}

impl AnalyzeSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
