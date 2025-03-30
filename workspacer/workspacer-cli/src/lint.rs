// ---------------- [ File: workspacer-cli/src/lint.rs ]
crate::ix!();

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
            // If user specifies `ws lint crate --crate <PATH>`
            LintSubcommand::Crate { crate_name } => {
                // We'll lint just that one crate by passing a manifest-path to cargo clippy.
                // We can reuse `run_with_workspace_and_crate_name` to ensure we have a valid workspace
                // containing that crate (and do a Git clean check if you want).
                // But note that `crate_name` here is a *PathBuf*, so the user might be specifying
                // the actual path (not the crate “name”). 
                // We'll assume we interpret it as a path on disk containing a Cargo.toml.
                // If you truly want to interpret it as the "name" in the workspace, see the note below.

                let crate_path_owned = crate_name.clone();
                info!("Linting single crate at path='{}'", crate_path_owned.display());

                // One approach: load a CrateHandle directly, run cargo clippy on that one crate's manifest-path:
                let crate_result = async {
                    let handle = CrateHandle::new(&crate_path_owned).await.map_err(|crate_err| {
                        error!(
                            "Failed to create CrateHandle for '{}': {:?}",
                            crate_path_owned.display(),
                            crate_err
                        );
                        WorkspaceError::CrateError(crate_err)
                    })?;

                    // We’ll call an internal function to run clippy with `--manifest-path=.../Cargo.toml`
                    let report = handle.run_linting().await?;
                    info!(
                        "Lint successful for crate='{}': success={} ",
                        handle.name(),
                        report.success()
                    );
                    println!("stdout:\n{}", report.stdout());
                    println!("stderr:\n{}", report.stderr());
                    Ok::<(), WorkspaceError>(())
                }
                .await;

                crate_result
            }

            // If user specifies `ws lint workspace --path <DIR>`
            LintSubcommand::Workspace { path } => {
                info!("Linting entire workspace at '{}'", path.display());

                // We can reuse our run_with_workspace(...) helper so that we automatically load the workspace,
                // optionally check Git cleanliness, etc. Then we call the `RunLinting::run_linting` trait
                // for the entire workspace. That’s how we do “cargo clippy” on the workspace root.
                run_with_workspace(Some(path.clone()), /*skip_git_check=*/false, move |ws| {
                    Box::pin(async move {
                        // We have a &mut Workspace<...>. 
                        // The trait `RunLinting for Workspace` is in the `workspacer-linting` crate:
                        let report = ws.run_linting().await?;

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
