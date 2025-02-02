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
use world_city_and_street_db_builder::*;
use tracing_setup::*;
use structopt::*;

fn main() -> Result<(),WorldCityAndStreetDbBuilderError> {
    configure_tracing();
    tracing::info!("starting WorldCityAndStreetDbBuilder REPL");
    repl_main()?;
    Ok(())
}
