// ---------------- [ File: workspacer-cli/src/bump.rs ]
crate::ix!();

/// Our top-level subcommand for `ws bump`.
/// We have three variants:
///
/// - **`Workspace`** => bump all crates in the workspace
/// - **`SingleCrate`** => only that one crate
/// - **`CrateAndDownstreams`** => that crate plus anything that depends on it
#[derive(Debug, StructOpt)]
pub enum BumpSubcommand {
    /// Bump the entire workspace
    #[structopt(name = "workspace")]
    Workspace(BumpWorkspaceCommand),

    /// Bump a single crate only (no downstream changes)
    #[structopt(name = "crate")]
    SingleCrate(BumpSingleCrateCommand),

    /// Bump a single crate and all crates that depend on it (recursively)
    #[structopt(name = "crate-downstreams")]
    CrateDownstreams(BumpCrateDownstreamsCommand),
}

impl BumpSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            BumpSubcommand::Workspace(cmd)        => cmd.run().await,
            BumpSubcommand::SingleCrate(cmd)      => cmd.run().await,
            BumpSubcommand::CrateDownstreams(cmd) => cmd.run().await,
        }
    }
}
