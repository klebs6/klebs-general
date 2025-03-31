// ---------------- [ File: workspacer-cli/src/pin.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum PinSubcommand {
    /// Pin wildcard dependencies in a single crate
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },

    /// Pin wildcard dependencies in an entire workspace
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl PinSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            // ----------------------------------------------------
            // 1) Single crate
            // ----------------------------------------------------
            PinSubcommand::Crate { crate_name } => {
                trace!("Pinning wildcard deps for single crate at '{}'", crate_name.display());

                // We'll use the `run_with_crate` helper so we load the crate,
                // optionally check Git, etc., then call `pin_all_wildcard_dependencies()`.
                run_with_crate(crate_name.clone(), /*skip_git_check=*/false, |handle| {
                    Box::pin(async move {

                        // The `PinAllWildcardDependencies` trait is implemented for `CrateHandle`
                        handle.pin_all_wildcard_dependencies().await.map_err(|crate_err| {
                            error!("Failed to pin wildcard deps in crate='{}': {:?}", handle.name(), crate_err);
                            WorkspaceError::CrateError(crate_err)
                        })?;

                        info!("Successfully pinned wildcard dependencies for crate='{}'", handle.name());
                        Ok(())
                    })
                })
                .await
            }

            // ----------------------------------------------------
            // 2) Entire workspace
            // ----------------------------------------------------
            PinSubcommand::Workspace { path } => {
                trace!("Pinning wildcard deps for workspace at '{}'", path.display());

                // We'll use `run_with_workspace` to load the workspace,
                // optionally check Git, then call `pin_all_wildcard_dependencies()`.
                run_with_workspace(Some(path.clone()), /*skip_git_check=*/false, |ws| {
                    Box::pin(async move {
                        // The `PinAllWildcardDependencies` trait is implemented for `Workspace<P,H>`
                        ws.pin_all_wildcard_dependencies().await.map_err(|we| {
                            error!(
                                "Failed to pin wildcard dependencies in workspace at '{}': {:?}",
                                ws.as_ref().display(),
                                we
                            );
                            we
                        })?;

                        info!("Successfully pinned wildcard dependencies in workspace at '{}'", ws.as_ref().display());
                        Ok(())
                    })
                })
                .await
            }
        }
    }
}
