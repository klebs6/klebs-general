// ---------------- [ File: src/bin/show_consolidated_crate_interface.rs ]
use workspacer_3p::*;
use structopt::*;
use workspacer_errors::*;
use workspacer_crate::*;
use workspacer_consolidate::*;
use workspacer_workspace::*;

/// Command line interface for the show_consolidated-crate-interface program
#[derive(StructOpt, Debug)]
#[structopt(name = "workspacer-show-consolidated-crate-interface")]
struct ConsolidateCrateInterfaceCli {
    /// Path to the crate to consolidate
    #[structopt(long)]
    path: Option<PathBuf>,

    /// Hide documentation.
    #[structopt(long)]
    no_show_docs: bool,

    /// Hide private items.
    #[structopt(long)]
    no_show_private_items: bool,

    /// Include test items.
    #[structopt(long)]
    show_test_items: bool,

    /// Hide function bodies.
    #[structopt(long)]
    no_show_fn_bodies: bool,

    /// Only test items.
    #[structopt(long)]
    only_show_test_items: bool,

    /// Include function bodies in tests.
    #[structopt(long)]
    show_fn_bodies_in_tests: bool,
}

impl ConsolidateCrateInterfaceCli {

    /// Production entry point for running the consolidation
    pub async fn run(&self) -> Result<(), WorkspaceError> {

        let mut path = PathBuf::from(".");

        if let Some(cli_path) = &self.path {
            path = cli_path.to_path_buf();
        }

        let single_crate = CrateHandle::new(&path)
            .await
            .map_err(WorkspaceError::CrateError)?;

        debug!("single_crate: {:#?}", single_crate);

        let mut consolidation_opts = ConsolidationOptions::new();

        if !self.no_show_docs {
            consolidation_opts = consolidation_opts.with_docs();
        }
        if !self.no_show_private_items {
            consolidation_opts = consolidation_opts.with_private_items();
        }
        if self.show_test_items {
            consolidation_opts = consolidation_opts.with_test_items();
        }
        if !self.no_show_fn_bodies {
            consolidation_opts = consolidation_opts.with_fn_bodies();
        }
        if self.only_show_test_items {
            consolidation_opts = consolidation_opts.with_only_test_items();
        }
        if self.show_fn_bodies_in_tests {
            consolidation_opts = consolidation_opts.with_fn_bodies_in_tests();
        }

        // Optionally logs helpful warnings
        consolidation_opts.validate();

        debug!("consolidation_options: {:#?}", consolidation_opts);

        let consolidated = single_crate
            .consolidate_crate_interface(&consolidation_opts)
            .await?;

        println!("\n\n{}", consolidated);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkspaceError> {
    configure_tracing();
    let cli = ConsolidateCrateInterfaceCli::from_args();
    cli.run().await
}

#[cfg(test)]
mod test_cli_flag_defaults {
    use super::*;
    use structopt::StructOpt;

    #[test]
    fn check_cli_default_values() {
        // We simulate calling this binary with no extra CLI flags
        let cli = ConsolidateCrateInterfaceCli::from_iter(&["test-binary"]);
        assert_eq!(cli.no_show_docs, false);
        assert_eq!(cli.no_show_private_items, false);
        assert_eq!(cli.show_test_items, false);
        assert_eq!(cli.no_show_fn_bodies, false);
        assert_eq!(cli.only_show_test_items, false);
        assert_eq!(cli.show_fn_bodies_in_tests, false);
    }
}
