// ---------------- [ File: workspacer-cli/src/publish.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum PublishSubcommand {
    /// Publish a single crate (assuming there's a local Cargo.toml)
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,

        #[structopt(long = "dry-run")]
        dry_run: bool,
    },

    /// Publish all crates in a workspace, in topological order
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,

        #[structopt(long = "dry-run")]
        dry_run: bool,
    },

    /// Publish one crate **and every workspace crate it depends on**
    CrateTree {
        /// Path to the workspace root (directory that contains the workspace‑level Cargo.toml)
        #[structopt(long = "path")]
        path: PathBuf,

        /// Name of the crate that should act as the tree’s *root*
        #[structopt(long = "root")]
        root: String,

        #[structopt(long = "dry-run")]
        dry_run: bool,
    },
}

impl PublishSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            PublishSubcommand::Crate { crate_name, dry_run } => {
                let dry_run = *dry_run;
                trace!("Publishing single crate at '{}'", crate_name.display());

                // Use `run_with_crate` to build a CrateHandle and optionally check Git, etc.
                // Then call the `TryPublish` trait method on it.
                run_with_crate(crate_name.clone(), /*skip_git_check=*/false, move |handle| {
                    Box::pin(async move {
                        // If you want a `dry_run` or other flags, you can pass them here.
                        // For example, handle.try_publish(dry_run).await. We’ll use false for a real publish.
                        handle.try_publish(dry_run).await.map_err(|crate_err| {
                            error!("Could not publish crate='{}': {:?}", handle.name(), crate_err);
                            WorkspaceError::CrateError(crate_err)
                        })?;

                        info!("Successfully published crate='{}'", handle.name());
                        Ok(())
                    })
                })
                .await
            }

            PublishSubcommand::Workspace { path, dry_run } => {
                let dry_run = *dry_run;
                trace!("Publishing entire workspace at '{}'", path.display());

                // Use `run_with_workspace` to load the workspace, check Git, etc. Then call `TryPublish`.
                run_with_workspace(Some(path.clone()), /*skip_git_check=*/false, move |ws| {
                    Box::pin(async move {
                        // If you want to do a "dry run" or pass flags, you can do that here.
                        // We'll do a real publish with `dry_run=false`.
                        ws.try_publish(dry_run).await.map_err(|err| {
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

            PublishSubcommand::CrateTree { path, root, dry_run } => {
                let dry_run = *dry_run;
                trace!(
                    "Publishing crate‑tree rooted at '{}' in workspace '{}'",
                    root,
                    path.display()
                );

                let root_clone = root.clone();
                run_with_workspace(Some(path.clone()), /*skip_git_check=*/false, move |ws| {
                    Box::pin(async move {
                        ws.try_publish_crate_tree(&root_clone, dry_run).await.map_err(|err| {
                            error!(
                                "Could not publish crate‑tree rooted at '{}' in workspace '{}': {:?}",
                                root_clone,
                                ws.as_ref().display(),
                                err
                            );
                            err
                        })
                    })
                })
                .await
            }
        }
    }
}
