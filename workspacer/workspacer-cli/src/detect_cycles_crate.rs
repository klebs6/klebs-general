// ---------------- [ File: workspacer-cli/src/detect_cycles_crate.rs ]
crate::ix!();

#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get="pub")]
pub struct DetectCyclesCrateCommand {
    /// Name of the crate we want to ensure is present
    #[structopt(long = "crate")]
    crate_name: String,

    /// If provided, custom workspace path
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// If true, we print extra logs
    #[structopt(long = "verbose")]
    verbose: bool,

    /// If true, skip the Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl DetectCyclesCrateCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // Clone out each needed field so that the closure can be `'static`:
        let crate_name_owned = self.crate_name.clone();
        let workspace_path_owned = self.workspace_path.clone();
        let skip_git_check_flag = self.skip_git_check;
        let verbose_flag = self.verbose;

        run_with_workspace_and_crate_name(
            workspace_path_owned,
            skip_git_check_flag,
            crate_name_owned,
            move |ws, name| {
                Box::pin(async move {
                    if verbose_flag {
                        info!("Verifying crate='{}' is in workspace, then checking for cycles ...", name);
                    }
                    // The crate is guaranteed present by `run_with_workspace_and_crate_name`.
                    // Next do the entire workspace check:
                    ws.detect_circular_dependencies().await.map_err(|err| {
                        error!("Failed detecting cycles in workspace: {:?}", err);
                        err
                    })?;

                    if verbose_flag {
                        info!("No circular dependencies detected in the workspace containing crate='{}'!", name);
                    } else {
                        println!("No circular dependencies found (crate='{}').", name);
                    }

                    Ok(())
                })
            },
        )
        .await
    }
}
