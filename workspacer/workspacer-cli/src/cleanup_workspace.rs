// ---------------- [ File: workspacer-cli/src/cleanup_workspace.rs ]
crate::ix!();

/// For **Workspace** subcommand usage: `ws cleanup workspace [--path ...] [--skip-git-check]`
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get="pub")]
pub struct CleanupWorkspaceCommand {
    /// If provided, we use this path as the workspace root
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// If true, we skip the Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl CleanupWorkspaceCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // 1) load the workspace, optionally ensure Git is clean
        run_with_workspace(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            move |ws| {
                Box::pin(async move {
                    // 2) call `ws.cleanup_workspace().await`
                    ws.cleanup_workspace().await?;
                    info!("Workspace successfully cleaned up (removed top-level target/, Cargo.lock, etc.)");
                    Ok(())
                })
            }
        ).await
    }
}
