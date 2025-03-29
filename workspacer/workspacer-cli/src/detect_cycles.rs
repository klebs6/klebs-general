// ---------------- [ File: workspacer-cli/src/detect_cycles.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum DetectCyclesSubcommand {
    /// Detect circular dependencies in a single crate’s workspace
    #[structopt(name = "crate")]
    Crate(DetectCyclesCrateCommand),

    /// Detect circular dependencies in the entire workspace
    #[structopt(name = "workspace")]
    Workspace(DetectCyclesWorkspaceCommand),
}

impl DetectCyclesSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            DetectCyclesSubcommand::Crate(cmd) => cmd.run().await,
            DetectCyclesSubcommand::Workspace(cmd) => cmd.run().await,
        }
    }
}
