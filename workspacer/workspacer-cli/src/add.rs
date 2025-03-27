crate::ix!();

/// Add a new crate by name
#[derive(Debug, StructOpt)]
pub enum AddSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: String,
    },
}

impl AddSubcommand {
    pub async fn run(&self) {
        todo!();
    }
}
