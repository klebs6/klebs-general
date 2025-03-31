// ---------------- [ File: workspacer-cli/src/format_all_imports.rs ]
crate::ix!();

/// Subcommand data for `ws format all-imports --path <dir> [--skip-git-check]`
#[derive(Debug, StructOpt)]
pub struct FormatAllImportsCommand {
    /// If provided, we use this directory as the workspace root instead of the current dir
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// Skip checking for a clean Git state if true
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl FormatAllImportsCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let ws_path = self.workspace_path.clone();
        let skip_flag = self.skip_git_check;

        // Use our standard "run_with_workspace" helper that doesn't need a crate name
        run_with_workspace(ws_path, skip_flag, move |ws| {
            Box::pin(async move {
                // The `SortAndFormatImports` trait is also implemented for Workspaces
                ws.sort_and_format_imports().await.map_err(|err| {
                    error!("Failed to format imports in entire workspace: {:?}", err);
                    err
                })?;

                info!("Successfully formatted imports in the entire workspace!");
                Ok(())
            })
        })
        .await
    }
}
