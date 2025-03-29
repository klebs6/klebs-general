// ---------------- [ File: workspacer-cli/src/check_publish_ready_workspace.rs ]
crate::ix!();

/// For checking the entire workspace, we do a simpler approach:
///   - optional `--path` for the workspace root
///   - `--skip-git-check` if desired
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get = "pub")]
pub struct CheckPublishReadyWorkspaceCommand {
    /// If provided, use this as the workspace root
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// Skip Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl CheckPublishReadyWorkspaceCommand {
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // We do `run_with_workspace` => obtains &mut Workspace => call `ws.ready_for_cargo_publish().await`
        run_with_workspace(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            |ws| {
                Box::pin(async move {
                    // This calls the library trait on the entire workspace
                    ws.ready_for_cargo_publish().await.map_err(|err| {
                        error!("Workspace is NOT ready for publish: {:?}", err);
                        err
                    })?;

                    info!("All crates in workspace are READY for publish!");
                    Ok(())
                })
            },
        ).await
    }
}
