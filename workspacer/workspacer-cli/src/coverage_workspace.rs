crate::ix!();

/// Subcommand for `ws coverage workspace [--path ...] [--skip-git-check]`
#[derive(Debug,StructOpt,Getters,Setters)]
#[getset(get="pub")]
pub struct CoverageWorkspaceCommand {
    /// optional workspace path
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// skip git check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl CoverageWorkspaceCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // We'll do `run_with_workspace(...)` => load => call `ws.run_tests_with_coverage()`.
        run_with_workspace(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            |ws| {
                Box::pin(async move {
                    let coverage_report = ws.run_tests_with_coverage().await?;
                    info!("Workspace coverage: {coverage_report:?}");

                    println!(
                        "Workspace coverage => {:.2}% (covered: {}, missed: {}, total: {})",
                        coverage_report.total_coverage(),
                        coverage_report.covered_lines(),
                        coverage_report.missed_lines(),
                        coverage_report.total_lines()
                    );

                    Ok(())
                })
            },
        )
        .await
    }
}
