// ---------------- [ File: workspacer-cli/src/add_crate.rs ]
crate::ix!();

#[derive(Getters,Setters,Debug,StructOpt)]
#[getset(get="pub")]
pub struct AddCrateCommand {
    /// The name of the new crate to add
    #[structopt(long = "crate")]
    crate_name: String,

    /// If provided, we use this path as the workspace root instead of the current directory
    #[structopt(long = "workspace")]
    workspace_path: Option<PathBuf>,

    /// If true, we skip the Git clean check (i.e., do not require a clean repo)
    #[structopt(long = "skip-git-check")]
    skip_git_check: bool,
}

impl AddCrateCommand {

    pub async fn run(&self) -> Result<(), WorkspaceError> {
        trace!(
            "AddSubcommand::Crate invoked with crate_name='{}', workspace_path='{:?}', skip_git_check={}",
            self.crate_name(),
            self.workspace_path(),
            self.skip_git_check(),
        );

        // We create an owned String from the crate_name field
        let crate_name_owned = self.crate_name().clone();

        // Now we pass that owned string into our helper
        run_with_workspace_and_crate_name(
            self.workspace_path().clone(),
            *self.skip_git_check(),
            crate_name_owned, // <-- pass the owned String
            |ws, new_crate_name| {
                Box::pin(async move {
                    info!("Now performing 'add new crate to workspace' for '{}'", new_crate_name);
                    ws.add_new_crate_to_workspace(new_crate_name).await?;
                    debug!("Successfully added crate='{}' via subcommand logic", new_crate_name);
                    Ok(())
                })
            },
        ).await?;

        Ok(())
    }
}
