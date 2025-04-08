crate::ix!();

/// Subroutine for handling `ws show crate-tree` subcommand: single crate plus its internal dependencies.
/// If `merge_crates` is true, merges them into one interface; otherwise prints them separately with dividers.
#[tracing::instrument(level = "trace", skip(flags))]
pub async fn show_crate_tree(flags: &ShowFlags) -> Result<String, WorkspaceError> {
    trace!("User chose subcommand: ws show crate-tree");
    let path = flags
        .path()
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    debug!("Expecting single crate at {:?}", path);

    // We'll accumulate all output in a single string.
    let mut output = String::new();

    // We do a manual check for single crate vs workspace:
    match Workspace::<PathBuf, CrateHandle>::new(&path).await {
        Ok(_ws) => {
            // This is actually a workspace, which we don't support for crate-tree
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
            // We'll manually create a CrateHandle, gather the main crate interface, then gather deps.
            let mut main_crate =
                CrateHandle::new(&path).await.map_err(WorkspaceError::CrateError)?;

            let main_name = main_crate.name();
            let info_line = format!("// ---------------- [ Main crate: {} ]\n", main_name);
            info!("{}", info_line.trim());

            let mut combined_cci = main_crate
                .consolidate_crate_interface(&ConsolidationOptions::from(flags))
                .await
                .map_err(WorkspaceError::CrateError)?;

            let dep_names = main_crate
                .internal_dependencies()
                .await
                .map_err(WorkspaceError::CrateError)?;
            info!(
                "Found {} internal deps in '{}': {:?}",
                dep_names.len(),
                main_name,
                dep_names
            );

            if *flags.merge_crates() {
                // Merge them all into one single combined interface
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
                if !sub_out.trim().is_empty() {
                    output.push_str(&info_line);
                    output.push_str(&sub_out);
                    output.push('\n');
                }
            } else {

                let main_out = flags.build_filtered_string(&combined_cci, &main_name);

                if !main_out.trim().is_empty() {
                    // Show the main crate first
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
                    let dep_info_line = format!("// ---------------- [ Dep crate: {} ]\n", dname);
                    info!("{}", dep_info_line.trim());

                    let dep_out = flags.build_filtered_string(&dep_cci, &dname);
                    if !dep_out.trim().is_empty() {
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

    Ok(output)
}
