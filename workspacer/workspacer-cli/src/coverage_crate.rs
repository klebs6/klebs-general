// ---------------- [ File: workspacer-cli/src/coverage_crate.rs ]
crate::ix!();

/// Subcommand for `ws coverage crate --crate <NAME> [--workspace ...] [--skip-git-check]`
#[derive(Debug,StructOpt,Getters,Setters)]
#[getset(get="pub")]
pub struct CoverageCrateCommand {
    /// crate name
    #[structopt(long = "crate")]
    crate_name: String,

    /// optional workspace path
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// skip git check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl CoverageCrateCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let crate_name_owned = self.crate_name().clone();

        run_with_workspace_and_crate_name(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            crate_name_owned,
            |ws, name| {
                Box::pin(async move {
                    // 1) find crate by name
                    let arc_crate = ws.find_crate_by_name(name).await.ok_or_else(|| {
                        error!("No crate named '{}' found in workspace for coverage subcommand", name);
                        CrateError::CrateNotFoundInWorkspace {
                            crate_name: name.to_string(),
                        }
                    })?;

                    // 2) lock handle
                    let handle = arc_crate.lock().await.clone();

                    // 3) run coverage for just this crate => from new trait
                    let coverage_report = handle.run_tests_with_coverage().await?;

                    info!(
                        "Coverage for crate='{}': total_coverage={}%, covered_lines={}/{}",
                        handle.name(),
                        coverage_report.total_coverage(),
                        coverage_report.covered_lines(),
                        coverage_report.total_lines(),
                    );

                    // optionally, print or store the coverage
                    println!(
                        "Coverage for crate '{}' => {:.2}% (covered: {}, missed: {}, total: {})",
                        handle.name(),
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
