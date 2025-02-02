// ---------------- [ File: src/repl_main.rs ]
crate::ix!();

/// A complete “main” function that:
///   1) Synchronously spawns a Tokio runtime,
///   2) Builds the DB for DC/MD/VA,
///   3) Launches a REPL for queries.
///
/// Usage:
///   cargo run --example dmv_repl
///
/// The PBF files will be downloaded/stored in `./pbf` directory.
/// The RocksDB will be created in `./dmv_db`.
pub fn repl_main() -> Result<(), WorldCityAndStreetDbBuilderError> {
    // We need a runtime because `build_dmv_database` calls async code.
    let rt = match Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create tokio runtime: {}", e);
            return Err(e.into());
        }
    };

    let db_path   = PathBuf::from("./dmv_db");
    let pbf_dir   = PathBuf::from("./pbf");

    // Build the database for DC/MD/VA:
    let db_result = rt.block_on(build_dmv_database(&db_path, &pbf_dir));
    let db = match db_result {
        Ok(db_arc) => db_arc,
        Err(e) => {
            eprintln!("Failed to build DMV database: {:?}", e);
            return Err(e);
        }
    };

    // Now run the simple REPL:
    match run_repl(db) {
        Ok(()) => {
            println!("REPL exited normally.");
        }
        Err(e) => {
            eprintln!("REPL error: {}", e);
        }
    }

    Ok(())
}

