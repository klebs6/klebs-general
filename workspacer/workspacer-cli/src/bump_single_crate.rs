// ---------------- [ File: workspacer-cli/src/bump_single_crate.rs ]
crate::ix!();

/// Bump just a single crate (no downstream references updated).
/// We use the `Bump` trait on the crate handle itself.
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get = "pub")]
pub struct BumpSingleCrateCommand {
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

impl BumpSingleCrateCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let crate_name_owned = self.crate_name().clone();
        let ReleaseArg(release_type) = self.release_arg().clone();

        // We'll do `run_with_workspace_and_crate_name` => load workspace => find crate => apply Bump
        run_with_workspace_and_crate_name(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            crate_name_owned,
            move |ws, found_crate_name| {
                Box::pin(async move {
                    let arc_crate = ws.find_crate_by_name(found_crate_name).await.ok_or_else(|| {
                        error!("No crate named '{}' found in workspace", found_crate_name);
                        CrateError::CrateNotFoundInWorkspace {
                            crate_name: found_crate_name.to_owned(),
                        }
                    })?;

                    // Lock the crate to get a mutable handle
                    let mut handle = arc_crate.lock().await.clone();
                    handle.bump(release_type.clone()).await.map_err(|err| {
                        error!(
                            "Failed to bump crate='{}' with release={:?}: {:?}",
                            found_crate_name, release_type, err
                        );
                        WorkspaceError::BumpError {
                            crate_path: handle.as_ref().join("Cargo.toml"),
                            source: Box::new(err),
                        }
                    })?;

                    info!(
                        "Successfully bumped single crate='{}' => release={:?}",
                        found_crate_name, release_type
                    );
                    Ok(())
                })
            },
        )
        .await
    }
}
