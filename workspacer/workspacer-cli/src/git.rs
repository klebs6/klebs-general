crate::ix!();

/// Subcommands for `ws git`
#[derive(Debug, StructOpt)]
pub enum GitSubcommand {
    /// Perform a git commit
    Commit,
}

impl GitSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
