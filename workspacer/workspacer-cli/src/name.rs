// ---------------- [ File: workspacer-cli/src/name.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum NameSubcommand {
    /// Name all files in a single crate (e.g., `--crate some/path/to/crate`)
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },

    /// Name all files in an entire workspace (e.g., `--path some/path/to/workspace`)
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl NameSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            // --------------------------------------
            // 1) Single Crate
            // --------------------------------------
            NameSubcommand::Crate { crate_name } => {
                trace!("Naming all .rs files in single crate at '{}'", crate_name.display());

                // We'll use our standard `run_with_crate` helper
                // to construct a CrateHandle, optionally check Git, then run our closure.
                run_with_crate(crate_name.clone(), /*skip_git_check=*/false, move |handle| {
                    Box::pin(async move {
                        // Using the `NameAllFiles` trait implemented for CrateHandle
                        handle.name_all_files().await.map_err(|crate_err| {
                            error!("Error naming all files in crate='{}': {:?}", handle.name(), crate_err);
                            // Wrap or convert into WorkspaceError as needed
                            WorkspaceError::CrateError(crate_err)
                        })?;

                        info!("Successfully named all files in crate='{}'", handle.name());
                        Ok(())
                    })
                })
                .await
            }

            // --------------------------------------
            // 2) Entire Workspace
            // --------------------------------------
            NameSubcommand::Workspace { path } => {
                trace!("Naming all .rs files in workspace at '{}'", path.display());

                // We'll use `run_with_workspace` so that we automatically load the workspace,
                // optionally check Git cleanliness, etc.
                run_with_workspace(Some(path.clone()), /*skip_git_check=*/false, move |ws| {
                    Box::pin(async move {
                        // The trait `NameAllFiles` is also impl’d for `Workspace<P,H>`.
                        ws.name_all_files().await.map_err(|we| {
                            error!("Error naming all files in workspace at '{}': {:?}", ws.as_ref().display(), we);
                            we
                        })?;

                        info!(
                            "Successfully named all .rs files in workspace at '{}'",
                            ws.as_ref().display()
                        );
                        Ok(())
                    })
                })
                .await
            }
        }
    }
}
