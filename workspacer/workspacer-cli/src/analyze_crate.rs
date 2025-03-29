// ---------------- [ File: workspacer-cli/src/analyze_crate.rs ]
crate::ix!();

/// For analyzing a single crate, we’ll require:
///  - `crate_name` (string) — we’ll interpret it as the crate’s name in the workspace
///  - `workspace_path` (optional) — or else current dir
///  - `skip_git_check` (bool) — skip checking for a clean Git state if desired
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get = "pub")]
pub struct AnalyzeCrateCommand {
    /// The name of the crate to analyze
    #[structopt(long = "crate")]
    crate_name: String,

    /// If provided, we use this as the workspace root
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// Skip Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl AnalyzeCrateCommand {
    #[tracing::instrument(level="trace", skip(self))]
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let crate_name_owned = self.crate_name().clone();

        // 1) We reuse our existing helper `run_with_workspace_and_crate_name`.
        //    That loads the workspace, optionally checks Git, etc.
        run_with_workspace_and_crate_name(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            crate_name_owned,
            // 2) Provide a closure that finds the crate, calls `CrateAnalysis::new(...)`,
            //    and then prints or logs the result.
            |ws, crate_name| {
                Box::pin(async move {
                    // a) find the crate by name
                    let arc_h = ws
                        .find_crate_by_name(crate_name)
                        .await
                        .ok_or_else(|| {
                            error!("No crate named '{}' found in workspace", crate_name);
                            CrateError::CrateNotFoundInWorkspace {
                                crate_name: crate_name.to_owned(),
                            }
                        })?;
                    // b) lock to get the real handle
                    let handle = arc_h.lock().await.clone();
                    
                    // c) create the crate analysis
                    let analysis = CrateAnalysis::new(&handle).await?;
                    info!("Crate analysis complete for '{}'\n{:#?}", crate_name,analysis);
                    Ok(())
                })
            },
        )
        .await
    }
}
