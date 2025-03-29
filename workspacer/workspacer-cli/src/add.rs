// ---------------- [ File: workspacer-cli/src/add.rs ]
crate::ix!();

#[derive(Debug,StructOpt)]
pub enum AddSubcommand {
    /// Add a new crate to the workspace
    #[structopt(name = "crate")]
    Crate(AddCrateCommand),

    /// Add an internal dependency (target-crate depends on dep-crate)
    #[structopt(name = "internal-dep")]
    InternalDep(AddInternalDepCommand),
}

/// Next, in your `AddSubcommand::run` method (where the lifetime error appeared),
/// simply clone the `crate_name` to pass as owned `String` to the helper.
impl AddSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        trace!("Entering AddSubcommand::run with {:?}", self);

        match self {
            AddSubcommand::Crate(cmd) => { cmd.run().await? }
            AddSubcommand::InternalDep(cmd)  => { cmd.run().await? }
        }

        Ok(())
    }
}
