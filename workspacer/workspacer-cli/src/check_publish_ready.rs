// ---------------- [ File: workspacer-cli/src/check_publish_ready.rs ]
crate::ix!();

/// We’ll extend `CheckPublishReadySubcommand` to have two variants:
///   - Crate => check just one crate
///   - Workspace => check all crates in the workspace
///
/// Then, for each variant, we define a small struct that implements a `.run()` method,
/// which uses the library trait `ReadyForCargoPublish` on either the crate handle or 
/// the entire workspace.

/// First, define the two variants.
#[derive(Debug, StructOpt)]
pub enum CheckPublishReadySubcommand {
    /// Check if a single crate is ready for publishing
    #[structopt(name = "crate")]
    Crate(CheckPublishReadyCrateCommand),

    /// Check if the entire workspace is ready for publishing
    #[structopt(name = "workspace")]
    Workspace(CheckPublishReadyWorkspaceCommand),
}

impl CheckPublishReadySubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            CheckPublishReadySubcommand::Crate(cmd) => cmd.run().await,
            CheckPublishReadySubcommand::Workspace(cmd) => cmd.run().await,
        }
    }
}
