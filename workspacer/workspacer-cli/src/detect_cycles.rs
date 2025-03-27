crate::ix!();

/// Detect cycles in dependencies
#[derive(Debug, StructOpt)]
pub struct DetectCyclesSubcommand {
    verbose: bool,
}

impl DetectCyclesSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
