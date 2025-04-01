// ---------------- [ File: workspacer-cli/src/prune.rs ]
crate::ix!();

/// The top-level `prune` subcommand has two variants:
///   - **Crate** => `ws prune crate --crate <NAME>` => calls `prune_invalid_category_slugs()` on that single crate
///   - **Workspace** => `ws prune workspace [--path <DIR>]` => calls `prune_invalid_category_slugs_from_members()` on the entire workspace
#[derive(Debug, StructOpt)]
pub enum PruneSubcommand {
    /// Prune categories/keywords in a single crate
    #[structopt(name = "crate")]
    Crate(PruneCrateCommand),

    /// Prune categories/keywords in the entire workspace
    #[structopt(name = "workspace")]
    Workspace(PruneWorkspaceCommand),
}

impl PruneSubcommand {
    /// Entrypoint for `ws prune ...`
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            PruneSubcommand::Crate(cmd) => cmd.run().await,
            PruneSubcommand::Workspace(cmd) => cmd.run().await,
        }
    }
}

/// Subcommand data for `ws prune crate --crate <NAME> [--workspace <dir>] [--skip-git-check]`
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get="pub")]
pub struct PruneCrateCommand {
    /// The name (or path) of the crate
    #[structopt(long = "crate")]
    crate_name: String,

    /// Optional path to the workspace root
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// If true, skip checking for a clean Git repo
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl PruneCrateCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        let crate_name_owned = self.crate_name().clone();

        // Use the existing helper that loads the workspace, 
        // finds the crate, checks Git cleanliness (if not skipped), etc.
        run_with_workspace_and_crate_name(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            crate_name_owned,
            |ws, crate_name_str| {
                Box::pin(async move {
                    // 1) find the crate
                    let arc_crate = ws.find_crate_by_name(crate_name_str).await
                        .ok_or_else(|| {
                            error!("No crate named '{}' found in workspace", crate_name_str);
                            CrateError::CrateNotFoundInWorkspace {
                                crate_name: crate_name_str.to_string(),
                            }
                        })?;
                    // 2) Lock the crate handle
                    let mut guard = arc_crate.lock().await;
                    // 3) Call `prune_invalid_category_slugs` on this one crate
                    let removed = guard.prune_invalid_category_slugs().await?;
                    
                    info!("Prune done for crate='{}': removed {} invalid category/keyword entries.", crate_name_str, removed);
                    println!("Removed {} invalid category or keyword entries from crate='{}'.", removed, crate_name_str);
                    Ok(())
                })
            },
        )
        .await
    }
}

/// Subcommand data for `ws prune workspace [--path <DIR>] [--skip-git-check]`
#[derive(Debug, StructOpt, Getters, Setters)]
#[getset(get="pub")]
pub struct PruneWorkspaceCommand {
    /// If provided, we use this path as the workspace root
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// If true, skip the Git clean check
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl PruneWorkspaceCommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // We'll do the same approach as e.g. FormatAllImportsCommand
        // => load the workspace, optionally ensure Git is clean, then call the trait.
        let ws_path = self.workspace_path.clone();
        let skip_flag = self.skip_git_check;

        run_with_workspace(ws_path, skip_flag, move |ws| {
            Box::pin(async move {
                // The `PruneInvalidCategorySlugsFromSubstructures` trait is implemented for Workspace
                let removed_total = ws.prune_invalid_category_slugs_from_members().await?;
                info!("Successfully pruned categories in the entire workspace, total removed={}", removed_total);
                println!("Removed {} invalid category/keyword entries across the entire workspace.", removed_total);
                Ok(())
            })
        })
        .await
    }
}
