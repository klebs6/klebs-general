// ---------------- [ File: src/bin/consolidate_crate_interface.rs ]
//! src/bin/consolidate_crate_interface.rs
use workspacer_3p::*;
use structopt::*;
use workspacer_interface::*;
use workspacer_crate::*;
use workspacer_consolidate::*;
use workspacer::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "consolidate-crate-interface")]
struct ConsolidateCrateInterfaceCli {
    #[structopt(long)]
    path: Option<PathBuf>,
}

impl ConsolidateCrateInterfaceCli {

    /// The production entry point
    pub async fn run(&self) -> Result<(), WorkspaceError> {

        let mut path = PathBuf::from(".");

        if let Some(cli_path) = &self.path {
            path = cli_path.to_path_buf();
        }

        let single_crate = CrateHandle::new(&path).await
            .map_err(|e| WorkspaceError::CrateError(e))?;

        info!("single_crate: {:#?}", single_crate);

        // could possibly extend these with cli flags, if desired
        let opts = ConsolidationOptions::new()
            .with_docs()
            //.with_private_items()
            .with_test_items()
            //.with_fn_bodies()
            //.with_only_test_items()
            .with_fn_bodies_in_tests()
            ;

        // optionally logs helpful warnings
        opts.validate(); 

        info!("consolidation_options: {:#?}", opts);

        let consolidated = single_crate.consolidate_crate_interface(&opts).await?;

        info!("\n\n{}", consolidated);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkspaceError> {
    configure_tracing();
    let cli = ConsolidateCrateInterfaceCli::from_args();
    cli.run().await
}
