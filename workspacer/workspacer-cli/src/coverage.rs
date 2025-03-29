// ---------------- [ File: workspacer-cli/src/coverage.rs ]
crate::ix!();

// 2) Now define the CoverageSubcommand with two variants: Crate(...) & Workspace(...)
//    We'll implement the subcommand with typical "run()" pattern: 
//    - For crate coverage => find the crate => run coverage for that crate only
//    - For workspace coverage => call `ws.run_tests_with_coverage()`
//
#[derive(Debug, StructOpt)]
pub enum CoverageSubcommand {
    /// Run test coverage for a single crate
    #[structopt(name = "crate")]
    Crate(CoverageCrateCommand),

    /// Run test coverage for the entire workspace
    #[structopt(name = "workspace")]
    Workspace(CoverageWorkspaceCommand),
}

impl CoverageSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            CoverageSubcommand::Crate(cmd) => cmd.run().await,
            CoverageSubcommand::Workspace(cmd) => cmd.run().await,
        }
    }
}
