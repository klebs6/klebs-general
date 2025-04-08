// ---------------- [ File: workspacer-cli/src/show.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum ShowSubcommand {
    /// Show info for a single crate only
    #[structopt(name = "crate")]
    Crate(ShowFlags),

    /// Show info for a single crate plus its internal deps,
    /// concatenating all consolidated interfaces into one final result
    #[structopt(name = "crate-tree")]
    CrateTree(ShowFlags),

    /// Show info for the entire workspace
    #[structopt(name = "workspace")]
    Workspace(ShowFlags),
}

impl ShowSubcommand {
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        trace!("Entering ShowSubcommand::run");

        let mut output = String::new();

        match self {
            ShowSubcommand::Crate(flags) => {
                info!("User chose subcommand: ws show crate");
                let path = flags.path.clone().unwrap_or_else(|| PathBuf::from("."));
                debug!("Path for single crate = {:?}", path);

                // We'll attempt to build a workspace first:
                match Workspace::<PathBuf, CrateHandle>::new(&path).await {
                    Ok(_ws) => {
                        error!(
                            "Found a workspace at {:?}, but subcommand=Crate requires a single crate",
                            path
                        );
                        return Err(WorkspaceError::InvalidWorkspace {
                            invalid_workspace_path: path,
                        });
                    }
                    Err(WorkspaceError::ActuallyInSingleCrate { path: single_crate_path }) => {
                        debug!("Confirmed single crate at {:?}", single_crate_path);

                        let mut handle = CrateHandle::new(&path)
                            .await
                            .map_err(WorkspaceError::CrateError)?;
                        let show_opts = flags_to_show_options(flags, false); // merge_crates = false
                        let cci_str = handle
                            .show_crate(&show_opts)
                            .await
                            .map_err(WorkspaceError::CrateError)?;

                        if cci_str.trim().is_empty() && show_opts.show_items_with_no_data() {
                            output.push_str("<no-data>\n");
                        } else {
                            output.push_str(&cci_str);
                        }
                    }
                    Err(e) => {
                        error!("Could not interpret path {:?} as single crate: {:?}", path, e);
                        return Err(e);
                    }
                }
            }
            ShowSubcommand::CrateTree(flags) => {
                info!("User chose subcommand: ws show crate-tree");
                let path = flags.path.clone().unwrap_or_else(|| PathBuf::from("."));
                debug!("Path for single crate-tree = {:?}", path);

                match Workspace::<PathBuf, CrateHandle>::new(&path).await {
                    Ok(_ws) => {
                        error!(
                            "Found a workspace at {:?}, but subcommand=CrateTree requires a single crate",
                            path
                        );
                        return Err(WorkspaceError::InvalidWorkspace {
                            invalid_workspace_path: path,
                        });
                    }
                    Err(WorkspaceError::ActuallyInSingleCrate { path: single_crate_path }) => {
                        debug!("Confirmed single crate at {:?}", single_crate_path);
                        let mut handle = CrateHandle::new(&path)
                            .await
                            .map_err(WorkspaceError::CrateError)?;
                        let show_opts = flags_to_show_options(flags, true); // merge_crates = true
                        let cci_str = handle
                            .show_crate(&show_opts)
                            .await
                            .map_err(WorkspaceError::CrateError)?;

                        if cci_str.trim().is_empty() && show_opts.show_items_with_no_data() {
                            output.push_str("<no-data>\n");
                        } else {
                            output.push_str(&cci_str);
                        }
                    }
                    Err(e) => {
                        error!("Could not interpret path {:?} as single crate: {:?}", path, e);
                        return Err(e);
                    }
                }
            }
            ShowSubcommand::Workspace(flags) => {
                info!("User chose subcommand: ws show workspace");
                let path = flags.path.clone().unwrap_or_else(|| PathBuf::from("."));
                debug!("Path for workspace = {:?}", path);

                match Workspace::<PathBuf, CrateHandle>::new(&path).await {
                    Ok(workspace) => {
                        debug!("Confirmed a valid workspace at {:?}", path);
                        let show_opts = flags_to_show_options(flags, false);
                        let ws_str = workspace.show_workspace(&show_opts).await?;
                        if ws_str.trim().is_empty() && show_opts.show_items_with_no_data() {
                            output.push_str("<no-data>\n");
                        } else {
                            output.push_str(&ws_str);
                        }
                    }
                    Err(WorkspaceError::ActuallyInSingleCrate { path: single_crate_path }) => {
                        error!(
                            "Found a single crate at {:?}, but subcommand=Workspace requires a full workspace",
                            single_crate_path
                        );
                        return Err(WorkspaceError::InvalidWorkspace {
                            invalid_workspace_path: single_crate_path,
                        });
                    }
                    Err(e) => {
                        error!("Could not interpret path {:?} as a workspace: {:?}", path, e);
                        return Err(e);
                    }
                }
            }
        }

        if output.trim().is_empty() {
            trace!("No data produced in ShowSubcommand::run");
        } else {
            println!("{}", output);
        }

        Ok(())
    }
}
