// ---------------- [ File: workspacer-cli/src/analyze_workspace.rs ]
crate::ix!();

/// For analyzing an entire workspace, we define a similar struct but without a crate name.
/// We just need to know (optionally) the workspace path and skip-git-check.
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get="pub")]
pub struct AnalyzeWorkspaceCommand {
    /// If provided, we use this as the workspace root
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// Skip Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl AnalyzeWorkspaceCommand {
    #[tracing::instrument(level="trace", skip(self))]
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // We define a simpler helper that loads the workspace
        // and optionally checks Git, but does not require a crate name.
        // We'll call it `run_with_workspace`.
        run_with_workspace(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            |ws| {
                Box::pin(async move {
                    // Here we do a full workspace analysis via `ws.analyze()`.
                    // That returns a `WorkspaceSizeAnalysis`.
                    let analysis = ws.analyze().await?;
                    info!("Workspace analysis complete.\n{:#?}",analysis);
                    Ok(())
                })
            },
        ).await
    }
}
