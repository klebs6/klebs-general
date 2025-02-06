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

#[cfg(test)]
mod build_dmv_database_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::sync::Mutex as StdMutex; // to track calls in test
    use std::collections::HashMap;

    /// We'll store a global mapping from Region => parse result
    /// in a std::sync::Mutex so we can decide if "download_and_parse_region" should succeed/fail/skip.
    static PARSE_BEHAVIOR: once_cell::sync::Lazy<StdMutex<HashMap<WorldRegion, Result<(), WorldCityAndStreetDbBuilderError>>>> 
        = once_cell::sync::Lazy::new(|| StdMutex::new(HashMap::new()));

    /// A test-specific override for `download_and_parse_region` that checks
    /// the global map to determine if it fails or succeeds for each region.
    /// If the region is not in the map, we default to success.
    async fn mock_download_and_parse_region(
        region: &WorldRegion,
        _pbf_dir: impl AsRef<Path>,
        db: &mut Database,
        write_to_storage: bool,
    ) -> Result<(), WorldCityAndStreetDbBuilderError> {
        let map = PARSE_BEHAVIOR.lock().unwrap();
        let result = map.get(region).cloned().unwrap_or(Ok(())); 
        drop(map);

        match result {
            Ok(_) => {
                // If parse is "Ok", we also mark region done. 
                // The real code does so, so we replicate it here:
                db.mark_region_done(region)?;
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    /// We'll define a local injection version of `build_dmv_database`,
    /// which calls our `mock_download_and_parse_region(...)` instead of the real one.
    /// This approach is common for testing tricky dependencies.
    async fn build_dmv_database_with_injection(
        db_path: impl AsRef<Path> + Send + Sync,
        pbf_dir: impl AsRef<Path> + Send + Sync,
    ) -> Result<Arc<Mutex<Database>>, WorldCityAndStreetDbBuilderError> 
    {
        // 1) Attempt to open or create DB
        let db = match Database::open(db_path) {
            Ok(db_arc) => db_arc,
            Err(e) => {
                return Err(WorldCityAndStreetDbBuilderError::DatabaseConstructionError(e));
            }
        };

        // 2) Lock the DB
        let mut db_guard = match db.lock() {
            Ok(g) => g,
            Err(_) => {
                return Err(WorldCityAndStreetDbBuilderError::DbLockError);
            }
        };

        // 3) For DC/MD/VA => call our mock parse
        for region in dmv_regions() {
            // If region_done => skip. We replicate the real logic:
            if db_guard.region_done(&region)? {
                continue;
            }
            // else parse
            mock_download_and_parse_region(&region, &pbf_dir, &mut db_guard, true).await?;
        }

        Ok(db)
    }

    // ---------------------------------------
    // Now the actual test scenarios
    // ---------------------------------------

    #[tokio::test]
    async fn test_build_dmv_database_db_open_fails() {
        // We'll pass an unwritable or invalid path so `Database::open(...)` fails.
        // On Unix, we might pick "/root/some_unwritable_dir" or a bogus path.
        #[cfg(unix)]
        {
            let invalid_path = PathBuf::from("/root/no_access_db");
            let pbf_dir = TempDir::new().unwrap();
            let result = build_dmv_database_with_injection(invalid_path, pbf_dir.path()).await;
            assert!(result.is_err());
            match result.err().unwrap() {
                WorldCityAndStreetDbBuilderError::DatabaseConstructionError(_) => {
                    // Good
                }
                other => panic!("Expected DatabaseConstructionError, got: {:?}", other),
            }
        }
    }

    #[tokio::test]
    async fn test_build_dmv_database_lock_poisoned() {
        // We open a valid DB, then forcibly poison the lock.
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("poison_db");
        {
            let db = Database::open(&db_path).unwrap();
            // Force poison
            let _ = std::panic::catch_unwind(|| {
                let guard = db.lock().unwrap();
                panic!("Intentional poison");
            });
        }

        // Now build => tries to lock => fails => DbLockError
        let pbf_dir = TempDir::new().unwrap();
        let result = build_dmv_database_with_injection(&db_path, pbf_dir.path()).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            WorldCityAndStreetDbBuilderError::DbLockError => { /* expected */ }
            other => panic!("Expected DbLockError, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_build_dmv_database_all_already_done() {
        // We open a DB and mark all dmv_regions => done upfront => so parse is skipped
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("done_db");
        {
            let db = Database::open(&db_path).unwrap();
            let mut db_guard = db.lock().unwrap();
            for r in dmv_regions() {
                db_guard.mark_region_done(&r).unwrap();
            }
        }

        // We'll also set parse behavior => success or anything
        {
            let mut map = PARSE_BEHAVIOR.lock().unwrap();
            for r in dmv_regions() {
                map.insert(r, Ok(()));
            }
        }

        let pbf_dir = TempDir::new().unwrap();
        let result = build_dmv_database_with_injection(&db_path, &pbf_dir).await;
        assert!(result.is_ok(), "All regions => already done => skip => success");
    }

    #[tokio::test]
    async fn test_build_dmv_database_parse_fails_for_one_region() {
        // We'll define parse behavior => MD => Ok, VA => fail, DC => Ok.
        {
            let mut map = PARSE_BEHAVIOR.lock().unwrap();
            // For demonstration, we identify MD, VA, DC from dmv_regions:
            let md = USRegion::UnitedState(UnitedState::Maryland).into();
            let va = USRegion::UnitedState(UnitedState::Virginia).into();
            let dc = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia).into();

            map.clear();
            map.insert(md, Ok(()));
            map.insert(va, Err(WorldCityAndStreetDbBuilderError::DownloadError(
                DownloadError::Unknown, // or any variant you have
            )));
            map.insert(dc, Ok(()));
        }

        // Now we open a fresh DB
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("partial_fail_db");
        let pbf_dir = TempDir::new().unwrap();

        let result = build_dmv_database_with_injection(&db_path, &pbf_dir).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            WorldCityAndStreetDbBuilderError::DownloadError(DownloadError::Unknown) => {
                // Good
            }
            other => panic!("Expected a DownloadError(Unknown), got {:?}", other),
        }

        // We can confirm that the code short-circuits => DC parse not attempted after VA fails 
        // (assuming your code does that in `download_and_parse_region`).
        // Or if it tries them in order, you handle it in `download_and_parse_region`.
    }

    #[tokio::test]
    async fn test_build_dmv_database_all_parse_success() {
        // We'll define parse behavior => all Ok.
        {
            let mut map = PARSE_BEHAVIOR.lock().unwrap();
            for r in dmv_regions() {
                map.insert(r, Ok(()));
            }
        }

        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("all_ok_db");
        let pbf_dir = TempDir::new().unwrap();

        let result = build_dmv_database_with_injection(&db_path, &pbf_dir).await;
        assert!(result.is_ok(), "All parse => success => entire build => success");

        // Possibly check that all regions are now done:
        let db = result.unwrap();
        let db_guard = db.lock().unwrap();
        for r in dmv_regions() {
            let done = db_guard.region_done(&r).unwrap();
            assert!(done, "Region {:?} should be marked done after successful parse", r);
        }
    }
}
