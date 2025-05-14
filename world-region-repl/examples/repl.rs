/// This example shows:
///   1) How to build a RocksDB database for the "DMV" region (DC, Maryland, Virginia)
///      using the existing `download_and_parse_region` functionality in this crate.
///   2) A simple command-line “REPL” (read-eval-print loop) that allows a user to search
///      or autocomplete addresses interactively from the built database.
///
/// It assumes you have all the core crate files (like `keys.rs`, `regions.rs`, etc.)
/// and the types in scope (`WorldRegion`, `Database`, `download_and_parse_region`,
/// `dmv_regions`, etc.).
///
/// Compile and run:
///
/// ```bash
/// cargo run --example dmv_repl
/// ```
///
/// This example is purely illustrative; in a real system you might
/// structure your binaries differently, add more robust auto-complete,
/// better error reporting, etc.

// ---------------- [ File: examples/repl.rs ]
use world_region_db::*;
use world_region_repl::*;
use tracing_setup::*;
use structopt::*;

use std::path::PathBuf;

/// The main entry: build or open DB, gather data, run REPL with fuzzy autocomplete and region toggling.
#[tokio::main]
async fn main() -> Result<(),WorldCityAndStreetDbBuilderError> {

    configure_tracing();
    tracing::info!("starting WorldCityAndStreetDbBuilder REPL");

    let db_path  = PathBuf::from("./tx_db");
    let pbf_path = PathBuf::from("./pbf");

    // 1) Build/Load DB (async):
    let db_arc = build_tx_database::<Database>(db_path, pbf_path).await?;

    // 2) Run interactive REPL (sync):
    if let Err(e) = run_interactive_repl(db_arc) {
        eprintln!("REPL error: {:?}", e);
    }

    Ok(())
}
