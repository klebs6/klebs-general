// ---------------- [ File: workspacer-cli/src/tree.rs ]
crate::ix!();

/// Print the crate or workspace tree
#[derive(Debug, StructOpt)]
pub struct TreeSubcommand {
    #[structopt(long)]
    verbose: bool,
    levels:  usize,
}

impl TreeSubcommand {
    pub async fn run(&self) -> Result<(),WorkspaceError> {
        todo!();
    }
}
