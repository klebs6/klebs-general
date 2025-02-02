// ---------------- [ File: src/dmv.rs ]
crate::ix!();

/// Builds (or updates) a RocksDB database with DC/MD/VA data, downloading
/// each region’s OSM PBF into `pbf_dir` if necessary.
///
/// Returns the opened database handle upon success.
pub async fn build_dmv_database(
    db_path: impl AsRef<Path> + Send + Sync,
    pbf_dir: impl AsRef<Path> + Send + Sync,
) -> Result<Arc<Mutex<Database>>, WorldCityAndStreetDbBuilderError> 
{
    // 1) Open (or create) the DB
    let db = match Database::open(db_path) {
        Ok(db_arc) => db_arc,
        Err(e) => {
            return Err(WorldCityAndStreetDbBuilderError::DatabaseConstructionError(e));
        }
    };

    // 2) For each DMV region, try to parse if not already done
    {
        let mut db_guard = match db.lock() {
            Ok(g) => g,
            Err(_) => {
                // Lock is poisoned
                return Err(WorldCityAndStreetDbBuilderError::DbLockError);
            }
        };

        // All DC/MD/VA
        let regions = dmv_regions();
        for region in regions {
            // This checks if region_done(...) first, so we can just always call it:
            download_and_parse_region(&region, &pbf_dir, &mut db_guard, true).await?;
        }
    }

    Ok(db)
}
