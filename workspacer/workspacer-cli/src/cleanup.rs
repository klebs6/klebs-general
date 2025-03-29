// ---------------- [ File: workspacer-cli/src/cleanup.rs ]
crate::ix!();

/// The top-level CleanupSubcommand has two variants:
///  - **Crate** => `cleanup crate --crate <NAME>` => calls `cleanup_crate()` on that crate
///  - **Workspace** => `cleanup workspace [--path <DIR>]` => calls `cleanup_workspace()` on the entire workspace
#[derive(Debug, StructOpt)]
pub enum CleanupSubcommand {
    /// Cleanup a single crate’s target/ directory, Cargo.lock, etc.
    #[structopt(name = "crate")]
    Crate(CleanupCrateCommand),

    /// Cleanup the entire workspace’s top-level target/ and Cargo.lock
    #[structopt(name = "workspace")]
    Workspace(CleanupWorkspaceCommand),
}

impl CleanupSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            CleanupSubcommand::Crate(cmd) => cmd.run().await,
            CleanupSubcommand::Workspace(cmd) => cmd.run().await,
        }
    }
}
