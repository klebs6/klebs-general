crate::ix!();

/// Lint the code
#[derive(Debug, StructOpt)]
pub enum LintSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl LintSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}

