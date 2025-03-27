crate::ix!();

/// Print the crate or workspace tree
#[derive(Debug, StructOpt)]
pub struct TreeSubcommand {
    verbose: bool,
    levels:  usize,
}

impl TreeSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
