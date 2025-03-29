crate::ix!();

/// Subcommand data for `ws format imports --crate <crate_name> [--workspace <dir>] [--skip-git-check]`
#[derive(Debug, StructOpt)]
pub struct FormatImportsCommand {
    /// The name of the crate to format
    #[structopt(long = "crate")]
    crate_name: String,

    /// If provided, we use this directory as the workspace root instead of the current dir
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// Skip checking for a clean Git state if true
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl FormatImportsCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let crate_name_owned = self.crate_name.clone();
        let skip_flag = self.skip_git_check;
        let ws_path = self.workspace_path.clone();

        // Use our standard helper that loads a workspace + optional crate name
        run_with_workspace_and_crate_name(
            ws_path,
            skip_flag,
            crate_name_owned,
            move |ws, the_crate_name| {
                Box::pin(async move {
                    // 1) Look up the crate handle
                    let arc_crate = ws
                        .find_crate_by_name(the_crate_name)
                        .await
                        .ok_or_else(|| {
                            error!("No crate named '{}' found in workspace", the_crate_name);
                            CrateError::CrateNotFoundInWorkspace {
                                crate_name: the_crate_name.to_owned(),
                            }
                        })?;

                    // 2) Lock and call sort_and_format_imports() on that crate
                    let mut handle = arc_crate.lock().await.clone();
                    handle
                        .sort_and_format_imports()
                        .await
                        .map_err(|e| {
                            error!(
                                "Failed to format imports for crate='{}': {:?}",
                                the_crate_name, e
                            );
                            WorkspaceError::CrateError(e)
                        })?;

                    info!("Successfully formatted imports for crate='{}'!", the_crate_name);
                    Ok(())
                })
            },
        )
        .await
    }
}
