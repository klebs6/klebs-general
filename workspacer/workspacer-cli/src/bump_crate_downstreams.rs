// ---------------- [ File: workspacer-cli/src/bump_crate_downstreams.rs ]
crate::ix!();

/// Bump a single crate and all crates that depend on it (recursively),
/// using `BumpCrateAndDownstreams::bump_crate_and_downstreams`.
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get="pub")]
pub struct BumpCrateDownstreamsCommand {
    /// The name of the crate to bump
    #[structopt(long = "crate")]
    crate_name: String,

    /// If provided, we use this path as the workspace root
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// Skip Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,

    /// The release type to apply (major, minor, patch, alpha[=N])
    #[structopt(long = "release", default_value = "patch")]
    release_arg: ReleaseArg,
}

impl BumpCrateDownstreamsCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let crate_name_owned = self.crate_name().clone();
        let ReleaseArg(release_type) = self.release_arg().clone();

        // We'll do `run_with_workspace_and_crate_name`, find the crate,
        // but *then* we call `bump_crate_and_downstreams` on the workspace.
        run_with_workspace_and_crate_name(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            crate_name_owned,
            move |ws, found_crate_name| {
                Box::pin(async move {
                    // find the crate
                    let arc_crate = ws.find_crate_by_name(found_crate_name).await.ok_or_else(|| {
                        error!("No crate named '{}' found in workspace", found_crate_name);
                        CrateError::CrateNotFoundInWorkspace {
                            crate_name: found_crate_name.to_owned(),
                        }
                    })?;

                    // Lock crate to get a local handle
                    let mut handle = arc_crate.lock().await.clone();

                    ws.bump_crate_and_downstreams(&mut handle, release_type.clone())
                        .await
                        .map_err(|err| {
                            error!(
                                "Failed to bump crate='{}' + downstreams with release={:?}: {:?}",
                                found_crate_name, release_type, err
                            );
                            err
                        })?;

                    info!(
                        "Successfully bumped crate='{}' + downstreams => release={:?}",
                        found_crate_name, release_type
                    );
                    Ok(())
                })
            },
        )
        .await
    }
}
