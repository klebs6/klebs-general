// ---------------- [ File: workspacer-cli/src/add_internal_dep.rs ]
crate::ix!();

/// 1) Define a new struct for the "internal-dep" subcommand arguments.
///    It needs `target_crate`, `dep_crate`, an optional `workspace_path`
///    and a `skip_git_check` flag, just like the other subcommands.
#[derive(Getters,Setters,Debug,StructOpt)]
#[getset(get="pub")]
pub struct AddInternalDepCommand {
    /// The crate that *will* depend on another
    #[structopt(long = "target-crate")]
    target_crate: String,

    /// The crate being depended upon
    #[structopt(long = "dep-crate")]
    dep_crate: String,

    /// If provided, we use this path as the workspace root instead of the current directory
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// If true, we skip the Git clean check (i.e., do not require a clean repo)
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl AddInternalDepCommand {
    /// This method drives the logic for adding a dependency from one crate to another.
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // We'll use a new helper function (shown below) that is nearly identical
        // to `run_with_workspace_and_crate_name`, but handles *two* crate names.
        //
        // Alternatively, you could define a more general `run_with_workspace` that
        // doesn't require a crate name at all, then do the logic inside the closure.
        // But here's the approach with a "two crate names" helper for consistency.
        //
        // Step 1) Clone them so we have owned Strings
        let target_name_owned = self.target_crate().clone();
        let dep_name_owned    = self.dep_crate().clone();

        // Step 2) Call the new helper
        run_with_workspace_and_two_crate_names(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            target_name_owned,
            dep_name_owned,
            |ws, target_name, dep_name| {
                Box::pin(async move {
                    info!(
                        "Now performing 'add internal dependency' => target='{}', dep='{}'",
                        target_name, dep_name
                    );

                    // 2a) Find the target crate by name
                    let maybe_target_crate = ws.find_crate_by_name(target_name).await;
                    let target_crate_arc = match maybe_target_crate {
                        Some(arc) => arc,
                        None => {
                            error!("No crate named '{}' found in workspace", target_name);
                            return Err(CrateError::CrateNotFoundInWorkspace {
                                crate_name: target_name.to_owned(),
                            }.into());
                        }
                    };

                    // 2b) Find the dep crate by name
                    let maybe_dep_crate = ws.find_crate_by_name(dep_name).await;
                    let dep_crate_arc = match maybe_dep_crate {
                        Some(arc) => arc,
                        None => {
                            error!("No crate named '{}' found in workspace", dep_name);
                            return Err(CrateError::CrateNotFoundInWorkspace {
                                crate_name: dep_name.to_owned(),
                            }.into());
                        }
                    };

                    // 3) Lock each crate handle to get an `H` (so we can pass references to .await calls)
                    let target_handle = target_crate_arc.lock().await.clone();
                    let dep_handle    = dep_crate_arc.lock().await.clone();

                    // 4) Perform the actual add_internal_dependency call
                    ws.add_internal_dependency(&target_handle, &dep_handle).await?;

                    info!(
                        "Successfully added an internal dep to crate='{}' on '{}'",
                        target_name, dep_name
                    );
                    Ok(())
                })
            },
        )
        .await
    }
}
