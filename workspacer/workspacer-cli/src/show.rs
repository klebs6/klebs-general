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

        // We'll accumulate user-facing output here:
        let mut output = String::new();

        match self {
            ShowSubcommand::Crate(flags) => {
                info!("User chose subcommand: ws show crate");
                let path = flags.path().clone().unwrap_or_else(|| PathBuf::from("."));
                trace!("Expecting single crate at {:?}", path);

                match Workspace::<PathBuf, CrateHandle>::new(&path).await {
                    Ok(_ws) => {
                        let msg = format!(
                            "Found a workspace at {:?}, but subcommand=Crate requires a single crate\n",
                            path
                        );
                        error!("{}", msg.trim());
                        return Err(WorkspaceError::InvalidWorkspace {
                            invalid_workspace_path: path,
                        });
                    }
                    Err(WorkspaceError::ActuallyInSingleCrate { path: single_crate_path }) => {
                        trace!("Confirmed single crate at {:?}", single_crate_path);
                        let mut single =
                            CrateHandle::new(&path).await.map_err(WorkspaceError::CrateError)?;

                        let main_cci = single
                            .consolidate_crate_interface(&ConsolidationOptions::from(flags))
                            .await
                            .map_err(WorkspaceError::CrateError)?;

                        let crate_name = single.name();
                        let info_line = format!("--- [Crate: {}] ---\n", crate_name);
                        info!("{}", info_line.trim());

                        // Build the text
                        let sub_out = flags.build_filtered_string(&main_cci, &crate_name);

                        // If sub_out is empty or whitespace
                        if sub_out.trim().is_empty() {
                            if *flags.show_items_with_no_data() {
                                // Then we show <no-data-for-crate>
                                output.push_str(&info_line);
                                output.push_str("<no-data-for-crate>\n\n");
                            }
                        } else {
                            output.push_str(&info_line);
                            output.push_str(&sub_out);
                            output.push('\n');
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
                let path = flags.path().clone().unwrap_or_else(|| PathBuf::from("."));
                trace!("Expecting single crate at {:?}", path);

                match Workspace::<PathBuf, CrateHandle>::new(&path).await {
                    Ok(_ws) => {
                        let msg = format!(
                            "Found a workspace at {:?}, but subcommand=CrateTree requires a single crate\n",
                            path
                        );
                        error!("{}", msg.trim());
                        return Err(WorkspaceError::InvalidWorkspace {
                            invalid_workspace_path: path,
                        });
                    }
                    Err(WorkspaceError::ActuallyInSingleCrate { path: single_crate_path }) => {
                        trace!("Confirmed single crate at {:?}", single_crate_path);

                        let mut main_crate =
                            CrateHandle::new(&path).await.map_err(WorkspaceError::CrateError)?;

                        let main_name = main_crate.name();
                        let info_line = format!("--- [Root crate: {}] ---\n", main_name);
                        info!("{}", info_line.trim());

                        let mut combined_cci = main_crate
                            .consolidate_crate_interface(&ConsolidationOptions::from(flags))
                            .await
                            .map_err(WorkspaceError::CrateError)?;

                        let dep_names = main_crate
                            .internal_dependencies()
                            .await
                            .map_err(WorkspaceError::CrateError)?;
                        info!("Found {} internal deps in '{}': {:?}", dep_names.len(), main_name, dep_names);

                        if *flags.merge_crates() {
                            trace!("Merging internal crates into the main interface");
                            for dep_name in dep_names {
                                let dep_path = match main_crate.root_dir_path_buf().parent() {
                                    Some(par) => par.join(&dep_name),
                                    None => {
                                        let msg = format!(
                                            "Cannot find parent dir for main crate path: {:?}",
                                            main_crate.root_dir_path_buf()
                                        );
                                        error!("{}", msg.trim());
                                        return Err(WorkspaceError::InvalidWorkspace {
                                            invalid_workspace_path: main_crate.root_dir_path_buf().clone(),
                                        });
                                    }
                                };
                                debug!("Attempting to load dep '{}' at {:?}", dep_name, dep_path);

                                let mut dep_crate =
                                    CrateHandle::new(&dep_path).await.map_err(WorkspaceError::CrateError)?;
                                let dep_cci = dep_crate
                                    .consolidate_crate_interface(&ConsolidationOptions::from(flags))
                                    .await
                                    .map_err(WorkspaceError::CrateError)?;

                                merge_in_place(&mut combined_cci, &dep_cci);
                                debug!("Merged interface for dep '{}' into combined_cci", dep_name);
                            }

                            // Build text for the merged interface
                            let sub_out = flags.build_filtered_string(&combined_cci, "merged-many");
                            if sub_out.trim().is_empty() {
                                // Possibly no data after merging everything
                                if *flags.show_items_with_no_data() {
                                    output.push_str(&info_line);
                                    output.push_str("<no-data-for-crate>\n\n");
                                }
                            } else {
                                output.push_str(&info_line);
                                output.push_str(&sub_out);
                                output.push('\n');
                            }
                        } else {
                            // Show main crate
                            let main_out = flags.build_filtered_string(&combined_cci, &main_name);
                            if main_out.trim().is_empty() {
                                if *flags.show_items_with_no_data() {
                                    output.push_str(&info_line);
                                    output.push_str("<no-data-for-crate>\n\n");
                                }
                            } else {
                                output.push_str(&info_line);
                                output.push_str(&main_out);
                                output.push('\n');
                            }

                            // Then each dep crate separately
                            for dep_name in dep_names {
                                let dep_path = match main_crate.root_dir_path_buf().parent() {
                                    Some(par) => par.join(&dep_name),
                                    None => {
                                        let msg = format!(
                                            "Cannot find parent dir for main crate path: {:?}",
                                            main_crate.root_dir_path_buf()
                                        );
                                        error!("{}", msg.trim());
                                        return Err(WorkspaceError::InvalidWorkspace {
                                            invalid_workspace_path: main_crate.root_dir_path_buf().clone(),
                                        });
                                    }
                                };
                                debug!("Attempting to load dep '{}' at {:?}", dep_name, dep_path);

                                let mut dep_crate =
                                    CrateHandle::new(&dep_path).await.map_err(WorkspaceError::CrateError)?;
                                let dep_cci = dep_crate
                                    .consolidate_crate_interface(&ConsolidationOptions::from(flags))
                                    .await
                                    .map_err(WorkspaceError::CrateError)?;

                                let dname = dep_crate.name();
                                let dep_info_line = format!("--- [Dep crate: {}] ---\n", dname);
                                info!("{}", dep_info_line.trim());

                                let dep_out = flags.build_filtered_string(&dep_cci, &dname);
                                if dep_out.trim().is_empty() {
                                    if *flags.show_items_with_no_data() {
                                        output.push_str(&dep_info_line);
                                        output.push_str("<no-data-for-crate>\n\n");
                                    }
                                } else {
                                    output.push_str(&dep_info_line);
                                    output.push_str(&dep_out);
                                    output.push('\n');
                                }
                            }
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
                let path = flags.path().clone().unwrap_or_else(|| PathBuf::from("."));
                trace!("Expecting workspace at {:?}", path);

                match Workspace::<PathBuf, CrateHandle>::new(&path).await {
                    Ok(workspace) => {
                        trace!("Confirmed a valid workspace at {:?}", path);
                        for crate_handle in workspace.crates() {
                            let mut guard = crate_handle.lock().await;
                            let cci = guard
                                .consolidate_crate_interface(&ConsolidationOptions::from(flags))
                                .await
                                .map_err(WorkspaceError::CrateError)?;
                            let cname = guard.name();
                            let info_line = format!("--- [Crate: {}] ---\n", cname);
                            info!("{}", info_line.trim());

                            let sub_out = flags.build_filtered_string(&cci, &cname);
                            if sub_out.trim().is_empty() {
                                if *flags.show_items_with_no_data() {
                                    output.push_str(&info_line);
                                    output.push_str("<no-data-for-crate>\n\n");
                                }
                            } else {
                                output.push_str(&info_line);
                                output.push_str(&sub_out);
                                output.push('\n');
                            }
                        }
                    }
                    Err(WorkspaceError::ActuallyInSingleCrate { path: single_crate_path }) => {
                        let msg = format!(
                            "Found a single crate at {:?}, but subcommand=Workspace requires a full workspace\n",
                            single_crate_path
                        );
                        error!("{}", msg.trim());
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

        // Finally, if the entire 'output' is empty, we can show `<no-data>`.
        // But only if show_items_with_no_data is set. Otherwise we skip entirely.
        if output.trim().is_empty() {
            if let Some(flags) = self.get_flags() {
                // This is a trick: we can't do `self` is ShowFlags. But we know each variant
                // has it. We define a small helper get_flags() that returns Option<&ShowFlags>.
                if *flags.show_items_with_no_data() {
                    println!("<no-data>");
                }
            }
        } else {
            // Print all of the user-facing text
            println!("{}", output);
        }

        Ok(())
    }

    /// Helper so we can get the ShowFlags no matter which variant we match.
    fn get_flags(&self) -> Option<&ShowFlags> {
        match self {
            ShowSubcommand::Crate(f) => Some(f),
            ShowSubcommand::CrateTree(f) => Some(f),
            ShowSubcommand::Workspace(f) => Some(f),
        }
    }
}
