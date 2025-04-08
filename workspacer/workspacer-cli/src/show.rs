// ---------------- [ File: workspacer-cli/src/show.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum ShowSubcommand {
    /// Show info for a single crate only
    #[structopt(name = "crate")]
    Crate(ShowFlags),

    /// Show info for a single crate plus its internal deps,
    /// concatenating all consolidated interfaces into one final result
    #[structopt(name = "crate-tree")]
    CrateTree(ShowFlags),

    /// Show info for the entire workspace
    #[structopt(name = "workspace")]
    Workspace(ShowFlags),
}

impl ShowSubcommand {
    /// Runs the show subcommand. We dispatch to one of the subroutines below, which replicate
    /// the old CLI logic: 
    /// - "crate" => single crate only 
    /// - "crate-tree" => single crate plus all its internal deps, either merged or separate 
    /// - "workspace" => entire workspace
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        trace!("Entering ShowSubcommand::run");
        let mut final_output = String::new();

        match self {
            ShowSubcommand::Crate(flags) => {
                let out = show_crate(flags).await?;
                final_output.push_str(&out);
            }
            ShowSubcommand::CrateTree(flags) => {
                let out = show_crate_tree(flags).await?;
                final_output.push_str(&out);
            }
            ShowSubcommand::Workspace(flags) => {
                let out = show_workspace(flags).await?;
                final_output.push_str(&out);
            }
        }

        if final_output.trim().is_empty() {
            trace!("No data produced in ShowSubcommand::run");
        } else {
            println!("{}", final_output);
        }

        Ok(())
    }
}
