// ---------------- [ File: workspacer-cli/src/check_publish_ready_crate.rs ]
crate::ix!();

/// For checking a single crate, we use the typical pattern of:
///   - `--crate <name>` to identify the crate
///   - optional `--workspace <path>` to specify the workspace root
///   - `--skip-git-check` to skip ensuring a clean Git state
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get = "pub")]
pub struct CheckPublishReadyCrateCommand {
    /// The name of the crate to check
    #[structopt(long = "crate")]
    crate_name: String,

    /// If provided, we use this path as the workspace root instead of the current directory
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// If true, we skip the Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl CheckPublishReadyCrateCommand {
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let crate_name_owned = self.crate_name().clone();

        // We do our usual pattern: load the workspace, optionally check Git, validate integrity, etc.
        run_with_workspace_and_crate_name(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            crate_name_owned,
            |ws, name| {
                Box::pin(async move {
                    // 1) find the crate by name
                    let arc_crate = ws.find_crate_by_name(name).await.ok_or_else(|| {
                        error!("No crate named '{}' found in workspace", name);
                        CrateError::CrateNotFoundInWorkspace {
                            crate_name: name.to_owned(),
                        }
                    })?;

                    // 2) lock the crate handle
                    let handle = arc_crate.lock().await.clone();

                    // 3) call `handle.ready_for_cargo_publish().await`
                    handle.ready_for_cargo_publish().await.map_err(|cr_err| {
                        error!("Crate '{}' not ready for publish: {:?}", name, cr_err);
                        WorkspaceError::CrateError(cr_err)
                    })?;

                    info!("Crate '{}' is READY for publishing!", name);
                    Ok(())
                })
            },
        )
        .await
    }
}
