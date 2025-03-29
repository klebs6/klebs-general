// ---------------- [ File: workspacer-cli/src/bump_workspace.rs ]
crate::ix!();

/// Bump all crates in the workspace at once. We use `BumpAll::bump_all`.
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get = "pub")]
pub struct BumpWorkspaceCommand {
    /// If provided, we use this path as the workspace root
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// Skip Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,

    /// The release type to apply (major, minor, patch, alpha[=N])
    #[structopt(long = "release", default_value = "patch")]
    release_arg: ReleaseArg,
}

impl BumpWorkspaceCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let ReleaseArg(release_type) = self.release_arg().clone();
        // We'll do `run_with_workspace` => load the workspace => call `bump_all(release_type)`
        run_with_workspace(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            move |ws| {
                Box::pin(async move {
                    // `ws` is a &mut Workspace<...>
                    // We call `BumpAll::bump_all` on it
                    ws.bump_all(release_type.clone()).await.map_err(|bump_err| {
                        // Wrap or pass along as a workspace error
                        error!("Failed to bump all crates in workspace: {:?}", bump_err);
                        WorkspaceError::from(bump_err)
                    })?;

                    info!("Successfully bumped entire workspace with release={:?}", release_type);
                    Ok(())
                })
            },
        )
        .await
    }
}
