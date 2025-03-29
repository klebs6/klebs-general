crate::ix!();

/// For **Crate** subcommand usage: `ws cleanup crate --crate MY_CRATE [--workspace ...] [--skip-git-check]`
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get="pub")]
pub struct CleanupCrateCommand {
    /// The name of the crate to clean up
    #[structopt(long = "crate")]
    crate_name: String,

    /// If provided, we use this path as the workspace root instead of the current directory
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// If true, we skip the Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl CleanupCrateCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let crate_name_owned = self.crate_name().clone();

        // 1) Load the workspace, optionally ensure Git is clean,
        //    find the crate, and pass it to a closure that calls `cleanup_crate()`.
        run_with_workspace_and_crate_name(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            crate_name_owned,
            |ws, name| {
                Box::pin(async move {
                    // a) find the crate by name
                    let arc_crate = ws.find_crate_by_name(name).await.ok_or_else(|| {
                        error!("No crate named '{}' found in workspace", name);
                        CrateError::CrateNotFoundInWorkspace {
                            crate_name: name.to_owned(),
                        }
                    })?;

                    // b) lock the crate handle
                    let handle = arc_crate.lock().await.clone();

                    // c) call `handle.cleanup_crate().await`
                    handle.cleanup_crate().await?;

                    info!("Crate '{}' cleaned up (removed target/, Cargo.lock if present).", name);
                    Ok(())
                })
            },
        )
        .await
    }
}
