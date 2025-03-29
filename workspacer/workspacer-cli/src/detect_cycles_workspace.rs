crate::ix!();

#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get="pub")]
pub struct DetectCyclesWorkspaceCommand {
    /// If provided, custom workspace path
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// If true, we print extra logs
    #[structopt(long = "verbose")]
    verbose: bool,

    /// If true, skip the Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl DetectCyclesWorkspaceCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // Again, copy out fields
        let workspace_path_owned = self.workspace_path.clone();
        let verbose_flag = self.verbose;
        let skip_check = self.skip_git_check;

        run_with_workspace(
            workspace_path_owned,
            skip_check,
            move |ws| {
                Box::pin(async move {
                    if verbose_flag {
                        info!("Checking for circular dependencies across the entire workspace ...");
                    }

                    ws.detect_circular_dependencies().await.map_err(|err| {
                        error!("Failed detecting cycles in workspace: {:?}", err);
                        err
                    })?;

                    if verbose_flag {
                        info!("No circular dependencies found in this workspace!");
                    } else {
                        println!("No circular dependencies found.");
                    }
                    Ok(())
                })
            },
        )
        .await
    }
}
