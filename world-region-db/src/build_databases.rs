// ---------------- [ File: src/dmv.rs ]
crate::ix!();

/// Builds (or updates) a RocksDB database with California data, downloading
/// the OSM PBF into `pbf_dir` if necessary.
///
/// Returns the opened database handle upon success.
pub async fn build_ca_database<I: StorageInterface>(
    db_path: impl AsRef<Path> + Send + Sync,
    pbf_dir: impl AsRef<Path> + Send + Sync,
) -> Result<Arc<Mutex<I>>, WorldCityAndStreetDbBuilderError>
{
    trace!("Attempting to open or create database at {:?}", db_path.as_ref());
    let db = I::open(db_path)?;

    {
        let mut db_guard = match db.lock() {
            Ok(g) => g,
            Err(_) => {
                error!("Database lock poisoned for California build operation");
                return Err(WorldCityAndStreetDbBuilderError::DbLockError);
            }
        };

        info!("Loading California regions");
        let regions = ca_regions();
        for region in regions {
            info!("Downloading and parsing region: {:?}", region);
            download_and_parse_region(&region, &pbf_dir, &mut *db_guard, true).await?;
            debug!("Finished region: {:?}", region);
        }
    }

    info!("Successfully built/updated California database");
    Ok(db)
}

/// Builds (or updates) a RocksDB database with Texas data, downloading
/// the OSM PBF into `pbf_dir` if necessary.
///
/// Returns the opened database handle upon success.
pub async fn build_tx_database<I: StorageInterface>(
    db_path: impl AsRef<Path> + Send + Sync,
    pbf_dir: impl AsRef<Path> + Send + Sync,
) -> Result<Arc<Mutex<I>>, WorldCityAndStreetDbBuilderError>
{
    trace!("Attempting to open or create database at {:?}", db_path.as_ref());
    let db = I::open(db_path)?;

    {
        let mut db_guard = match db.lock() {
            Ok(g) => g,
            Err(_) => {
                error!("Database lock poisoned for Texas build operation");
                return Err(WorldCityAndStreetDbBuilderError::DbLockError);
            }
        };

        info!("Loading Texas regions");
        let regions = tx_regions();
        for region in regions {
            info!("Downloading and parsing region: {:?}", region);
            download_and_parse_region(&region, &pbf_dir, &mut *db_guard, true).await?;
            debug!("Finished region: {:?}", region);
        }
    }

    info!("Successfully built/updated Texas database");
    Ok(db)
}

/// Builds (or updates) a RocksDB database with DC/MD/VA data, downloading
/// each region’s OSM PBF into `pbf_dir` if necessary.
///
/// Returns the opened database handle upon success.
pub async fn build_dmv_database<I:StorageInterface>(
    db_path: impl AsRef<Path> + Send + Sync,
    pbf_dir: impl AsRef<Path> + Send + Sync,
) -> Result<Arc<Mutex<I>>, WorldCityAndStreetDbBuilderError> 
{
    // 1) Open (or create) the DB
    let db = I::open(db_path)?;

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
            download_and_parse_region(&region, &pbf_dir, &mut *db_guard, true).await?;
        }
    }

    Ok(db)
}

/// Builds (or updates) a RocksDB database with DC/MD/VA data, downloading
/// each region’s OSM PBF into `pbf_dir` if necessary.
///
/// Returns the opened database handle upon success.
pub async fn build_full_database<I:StorageInterface>(
    db_path: impl AsRef<Path> + Send + Sync,
    pbf_dir: impl AsRef<Path> + Send + Sync,
) -> Result<Arc<Mutex<I>>, WorldCityAndStreetDbBuilderError> 
{
    // 1) Open (or create) the DB
    let db = I::open(db_path)?;

    // 2) For each DMV region, try to parse if not already done
    {
        let mut db_guard = match db.lock() {
            Ok(g) => g,
            Err(_) => {
                // Lock is poisoned
                return Err(WorldCityAndStreetDbBuilderError::DbLockError);
            }
        };

        let regions = known_regions();
        for region in regions {
            // This checks if region_done(...) first, so we can just always call it:
            download_and_parse_region(&region, &pbf_dir, &mut *db_guard, true).await?;
        }
    }

    Ok(db)
}

/// Builds (or updates) a RocksDB database with DC/MD/VA data, downloading
/// each region’s OSM PBF into `pbf_dir` if necessary.
///
/// Returns the opened database handle upon success.
pub async fn build_va_database<I:StorageInterface>(
    db_path: impl AsRef<Path> + Send + Sync,
    pbf_dir: impl AsRef<Path> + Send + Sync,
) -> Result<Arc<Mutex<I>>, WorldCityAndStreetDbBuilderError> 
{
    // 1) Open (or create) the DB
    let db = I::open(db_path)?;

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
        let regions = va_regions();
        for region in regions {
            // This checks if region_done(...) first, so we can just always call it:
            download_and_parse_region(&region, &pbf_dir, &mut *db_guard, true).await?;
        }
    }

    Ok(db)
}

#[cfg(test)]
mod build_database_tests {
    use super::*;

    /// Helper function that, for each region in the given slice, creates an empty tiny OSM PBF file in `pbf_dir`
    /// with the expected filename. Here we use `create_tiny_osm_pbf` (which produces a file without housenumber).
    async fn create_dummy_pbf_files_for_regions(pbf_dir: &PathBuf, regions: &[WorldRegion]) -> Result<(),OsmPbfParseError> {
        for region in regions {
            // The expected filename is built using our helper.
            let file_path = expected_filename_for_region(pbf_dir, region.download_link())?;
            // Call the tiny pbf file creator.
            // (You might choose the variant with or without housenumber.)
            create_tiny_osm_pbf(&file_path).await?;
        }
        Ok(())
    }

    /// Checks if a region is marked as "done" in the database.
    /// In our implementation, this means that the DB contains a key like "META:REGION_DONE:<abbr>"
    fn region_is_done<I:StorageInterface>(db: &I, region: &WorldRegion) 
        -> bool 
    {
        let meta_key = crate::meta_key::MetaKeyForRegion::from(*region);
        db.get(meta_key.key().as_bytes()).unwrap().is_some()
    }

    #[traced_test]
    #[disable]
    async fn test_build_ca_database_success() {
        info!("Testing build_ca_database with tiny PBF files for California");
        let db_temp  = TempDir::new().expect("Could not create temporary directory for DB");
        let pbf_temp = TempDir::new().expect("Could not create temporary directory for PBF files");

        let db_path  = db_temp.path().to_path_buf();
        let pbf_dir  = pbf_temp.path().to_path_buf();

        let regions = ca_regions();
        trace!("Regions for California: {:?}", regions);

        // Create tiny OSM PBFs for each region
        create_dummy_pbf_files_for_regions(&pbf_dir, &regions)
            .await
            .expect("Failed to create dummy PBF files for California");

        let db_result = build_ca_database(db_path.clone(), pbf_dir.clone()).await;
        assert!(db_result.is_ok(), "build_ca_database should succeed");
        let db = db_result.unwrap();

        {
            let db_guard = db.lock().unwrap();
            for region in &regions {
                assert!(
                    region_is_done(&db_guard, region),
                    "Region {} should be marked done in the DB",
                    region.abbreviation()
                );
            }
        }
    }

    #[tokio::test]
    async fn test_build_ca_database_failure_on_invalid_db_path() {
        info!("Testing build_ca_database failure with invalid DB path");
        let pbf_temp = TempDir::new().expect("Could not create temporary directory for PBF files");
        let pbf_dir = pbf_temp.path().to_path_buf();

        // Create a temporary file and use its path as the DB path (invalid).
        let invalid_db_file = TempDir::new()
            .expect("Failed to create temp dir")
            .into_path()
            .join("not_a_dir.txt");
        std::fs::write(&invalid_db_file, b"this is a file, not a directory")
            .expect("Failed to write to file");

        let db_result = build_ca_database::<Database>(invalid_db_file, pbf_dir).await;
        assert!(
            db_result.is_err(),
            "build_ca_database should fail on an invalid DB path"
        );
    }

    #[traced_test]
    #[disable]
    async fn test_build_dmv_database_success() {

        // Create temporary directories for the DB and for PBF files.
        let db_temp  = TempDir::new().expect("Could not create temporary directory for DB");
        let pbf_temp = TempDir::new().expect("Could not create temporary directory for PBF files");

        let db_path  = db_temp.path().to_path_buf();
        let pbf_dir  = pbf_temp.path().to_path_buf();

        // Get the DMV regions (e.g. Maryland, Virginia, DC)
        let regions = dmv_regions();

        // For each region, create a tiny OSM PBF file with the expected filename.
        create_dummy_pbf_files_for_regions(&pbf_dir, &regions)
            .await
            .expect("Failed to create dummy PBF files");

        // Now call build_dmv_database.
        let db_result = build_dmv_database(db_path.clone(), pbf_dir.clone()).await;
        assert!(db_result.is_ok(), "build_dmv_database should succeed");
        let db = db_result.unwrap();

        // Verify that for each region, the DB is marked as done.
        {
            let db_guard = db.lock().unwrap();
            for region in &regions {
                assert!(
                    region_is_done(&db_guard, region),
                    "Region {} should be marked done in the DB",
                    region.abbreviation()
                );
            }
        }
    }

    #[tokio::test]
    async fn test_build_dmv_database_failure_on_invalid_db_path() {
        // Test an error case by passing an invalid (non-writable) DB path.
        // For example, if we pass a file (instead of a directory) or a directory that doesn't exist.
        // (This test may be platform‐dependent.)
        let pbf_temp = TempDir::new().expect("Could not create temporary directory for PBF files");
        let pbf_dir = pbf_temp.path().to_path_buf();

        // Create a temporary file and use its path as the DB path.
        let invalid_db_file = TempDir::new().expect("Failed to create temp dir")
            .into_path()
            .join("not_a_dir.txt");
        // Write something into the file so it exists.
        std::fs::write(&invalid_db_file, b"this is a file, not a directory").expect("Failed to write to file");

        // Call build_dmv_database. It should fail with a DatabaseConstructionError.
        let db_result = build_dmv_database::<Database>(invalid_db_file, pbf_dir).await;
        assert!(db_result.is_err(), "build_dmv_database should fail on an invalid DB path");
    }

    // (Additional failure cases such as a poisoned lock might be tested in an integration environment.)
}
