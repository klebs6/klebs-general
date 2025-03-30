// ---------------- [ File: workspacer-cli/src/lint.rs ]
crate::ix!();

/// Now we can refactor our LintSubcommand to use `run_with_crate` for the `Crate` variant:
#[derive(Debug, StructOpt)]
pub enum LintSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl LintSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        trace!("Entering LintSubcommand::run with {:?}", self);

        match self {
            LintSubcommand::Crate { crate_name } => {
                info!("Linting single crate at path='{}'", crate_name.display());

                // We call our new `run_with_crate` function:
                run_with_crate(crate_name.clone(), false, move |handle| {
                    Box::pin(async move {
                        // Inside this closure, we have a &CrateHandle to do the lint:
                        let report = handle.run_linting().await.map_err(|lint_err| {
                            error!(
                                "Linting error for crate='{}': {:?}",
                                handle.name(),
                                lint_err
                            );
                            WorkspaceError::LintingError(lint_err)
                        })?;

                        info!(
                            "Lint successful for crate='{}': success={}",
                            handle.name(),
                            report.success()
                        );
                        println!("stdout:\n{}", report.stdout());
                        println!("stderr:\n{}", report.stderr());
                        Ok(())
                    })
                })
                .await
            }

            LintSubcommand::Workspace { path } => {
                info!("Linting entire workspace at '{}'", path.display());

                // We can reuse our existing `run_with_workspace` helper
                // which loads the entire workspace, checks Git, etc.
                run_with_workspace(Some(path.clone()), false, move |ws| {
                    Box::pin(async move {
                        let report = ws.run_linting().await.map_err(|lint_err| {
                            error!("Workspace linting failed: {:?}", lint_err);
                            WorkspaceError::LintingError(lint_err)
                        })?;

                        info!(
                            "Workspace lint success?={}, stdout len={}, stderr len={}",
                            report.success(),
                            report.stdout().len(),
                            report.stderr().len()
                        );
                        println!("Lint STDOUT:\n{}", report.stdout());
                        println!("Lint STDERR:\n{}", report.stderr());

                        Ok(())
                    })
                })
                .await
            }
        }
    }
}
