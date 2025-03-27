crate::ix!();

/// Subcommands for `ws get`
#[derive(Debug, StructOpt)]
pub enum GetSubcommand {
    /// Get lock-versions
    LockVersions,

    /// Get toml section with optional crate selection
    Toml {
        #[structopt(long = "section")]
        section: String,

        #[structopt(long = "crate")]
        crate_name: Option<String>,
    },
}

impl GetSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
