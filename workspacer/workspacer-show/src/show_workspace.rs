// ---------------- [ File: workspacer-show/src/show_workspace.rs ]
crate::ix!();

/// Subroutine for handling `ws show workspace` subcommand: multiple crates in a workspace.
/// Prints each crate individually with a divider line.
#[tracing::instrument(level = "trace", skip(flags))]
pub async fn show_workspace(flags: &ShowFlags) -> Result<String, WorkspaceError> {
    info!("User chose subcommand: ws show workspace");
    let path = flags
        .path()
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    debug!("Expecting workspace at {:?}", path);

    let mut output = String::new();

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
                let info_line = format!("// ---------------- [ File: {} ]\n", cname);
                info!("{}", info_line.trim());

                let sub_out = flags.build_filtered_string(&cci, &cname);

                if !sub_out.trim().is_empty() {
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

    Ok(output)
}
