// ---------------- [ File: workspacer-cli/src/publish.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum PublishSubcommand {
    /// Publish a single crate (assuming there's a local Cargo.toml)
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },

    /// Publish all crates in a workspace, in topological order
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl PublishSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            PublishSubcommand::Crate { crate_name } => {
                trace!("Publishing single crate at '{}'", crate_name.display());

                // Use `run_with_crate` to build a CrateHandle and optionally check Git, etc.
                // Then call the `TryPublish` trait method on it.
                run_with_crate(crate_name.clone(), /*skip_git_check=*/false, |handle| {
                    Box::pin(async move {
                        // If you want a `dry_run` or other flags, you can pass them here.
                        // For example, handle.try_publish(dry_run).await. We’ll use false for a real publish.
                        handle.try_publish(false).await.map_err(|crate_err| {
                            error!("Could not publish crate='{}': {:?}", handle.name(), crate_err);
                            WorkspaceError::CrateError(crate_err)
                        })?;

                        info!("Successfully published crate='{}'", handle.name());
                        Ok(())
                    })
                })
                .await
            }

            PublishSubcommand::Workspace { path } => {
                trace!("Publishing entire workspace at '{}'", path.display());

                // Use `run_with_workspace` to load the workspace, check Git, etc. Then call `TryPublish`.
                run_with_workspace(Some(path.clone()), /*skip_git_check=*/false, |ws| {
                    Box::pin(async move {
                        // If you want to do a "dry run" or pass flags, you can do that here.
                        // We'll do a real publish with `dry_run=false`.
                        ws.try_publish(false).await.map_err(|err| {
                            error!(
                                "Could not publish workspace at '{}': {:?}",
                                ws.as_ref().display(),
                                err
                            );
                            err
                        })?;

                        info!("Successfully published all crates in workspace at '{}'", ws.as_ref().display());
                        Ok(())
                    })
                })
                .await
            }
        }
    }
}
