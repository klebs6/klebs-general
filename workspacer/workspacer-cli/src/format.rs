// ---------------- [ File: workspacer-cli/src/format.rs ]
crate::ix!();

/// Format imports in all crates or a single crate
#[derive(Debug, StructOpt)]
pub enum FormatSubcommand {
    /// Format the imports in one specific crate
    Imports(FormatImportsCommand),

    /// Format the imports in all crates of this workspace
    AllImports(FormatAllImportsCommand),
}

impl FormatSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            FormatSubcommand::Imports(cmd) => cmd.run().await,
            FormatSubcommand::AllImports(cmd) => cmd.run().await,
        }
    }
}
