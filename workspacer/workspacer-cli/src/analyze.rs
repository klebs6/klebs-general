// ---------------- [ File: workspacer-cli/src/analyze.rs ]
crate::ix!();

/// First, let’s extend our `AnalyzeSubcommand` to handle two variants:
///  - **Crate**: analyze a single crate within the workspace
///  - **Workspace**: analyze the entire workspace
///
/// We’ll do so by defining two small “command structs” with appropriate fields
/// (e.g., optional `workspace_path`, a `skip_git_check` flag, and so on).
/// Then each variant will delegate to that struct’s `.run()` method.
#[derive(Debug, StructOpt)]
pub enum AnalyzeSubcommand {
    /// Analyze a single crate by name (must be part of a valid workspace).
    #[structopt(name = "crate")]
    Crate(AnalyzeCrateCommand),

    /// Analyze the entire workspace
    #[structopt(name = "workspace")]
    Workspace(AnalyzeWorkspaceCommand),
}

impl AnalyzeSubcommand {
    #[tracing::instrument(level="trace", skip(self))]
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            AnalyzeSubcommand::Crate(cmd) => {
                cmd.run().await?;
            }
            AnalyzeSubcommand::Workspace(cmd) => {
                cmd.run().await?;
            }
        }
        Ok(())
    }
}
