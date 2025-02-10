// ---------------- [ File: src/integrate_house_number_subranges_for_street.rs ]
// ---------------- [ File: src/integrate_house_number_subranges_for_street.rs ]
crate::ix!();

/// Integrates a list of new house‐number ranges into existing DB data for the given street,
/// then writes the merged result back to the DB. Logs warnings if load/store operations fail.
///
/// # Arguments
///
/// * `db`           - Database to update.
/// * `world_region` - Region context (used in key derivation).
/// * `street`       - The street whose house‐number ranges are being updated.
/// * `new_ranges`   - A list of new [`HouseNumberRange`] values to be merged.
///
/// # Returns
///
/// * `Ok(())` on success, or if partial failures occurred but we can continue.
/// * `Err(OsmPbfParseError)` if a critical error prevents further processing.
pub fn integrate_house_number_subranges_for_street<I:StoreHouseNumberRanges + LoadExistingStreetRanges>(
    db:           &mut I,
    world_region: &WorldRegion,
    street:       &StreetName,
    new_ranges:   &Vec<HouseNumberRange>,
) -> Result<(), OsmPbfParseError> {

    trace!(
        "integrate_house_number_subranges_for_street: street='{}', merging {} new ranges",
        street,
        new_ranges.len()
    );

    // Step 1) Load existing ranges (if any)
    let existing_opt = match db.load_existing_street_ranges(world_region, street) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "integrate_house_number_subranges_for_street: could not load existing ranges for street='{}': {:?}",
                street,
                e
            );
            None
        }
    };

    // Step 2) Merge
    let merged = merge_new_subranges(existing_opt.unwrap_or_default(), new_ranges);

    // Step 3) Store
    match db.store_house_number_ranges(world_region, street, &merged) {
        Ok(_) => {
            debug!(
                "integrate_house_number_subranges_for_street: successfully stored merged ranges for street='{}'",
                street
            );
        }
        Err(e) => {
            warn!(
                "integrate_house_number_subranges_for_street: could not store updated ranges for street='{}': {:?}",
                street, e
            );
        }
    }

    Ok(())
}

#[cfg(test)]
#[disable]
mod test_integrate_house_number_subranges_for_street {
    use super::*;
    use std::collections::BTreeSet;
    use tempfile::TempDir;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// Helper that creates a temporary RocksDB-backed `Database`.
    /// Returns `(db, tempdir)` so the tempdir is held until the test completes.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temporary directory");
        (db, temp_dir)
    }

    /// A utility to store house-number ranges under the street/region key directly.
    /// This allows us to simulate "existing" data in the DB.
    fn put_existing_ranges<I:StorageInterface>(
        db:     &mut I,
        region: &WorldRegion,
        street: &StreetName,
        ranges: Vec<HouseNumberRange>,
    ) {
        db.store_house_number_ranges(region, street, &ranges)
          .expect("Storing initial ranges should succeed in test");
    }

    /// A utility to read back the stored house-number ranges from the DB.
    /// Returns an empty vector if none found.
    fn get_stored_ranges<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        street: &StreetName
    ) -> Vec<HouseNumberRange> {
        match db.load_existing_street_ranges(region, street) {
            Ok(opt) => opt.unwrap_or_default(),
            Err(e) => {
                panic!("Unexpected load error in test: {:?}", e);
            }
        }
    }

    /// Simplifies constructing a `HouseNumberRange(start,end)`.
    fn range(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// Returns a dummy region + street for testing. 
    /// Adjust as desired if you prefer different region or street names.
    fn test_region_and_street() -> (WorldRegion, StreetName) {
        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let street = StreetName::new("Test Street").unwrap();
        (region, street)
    }

    #[traced_test]
    fn test_no_existing_ranges_new_ranges_simple() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let (region, street) = test_region_and_street();
        let new = vec![range(10, 15), range(20, 25)];

        // No existing ranges -> should simply store new ones
        integrate_house_number_subranges_for_street(&mut db_guard, &region, &street, &new)
            .expect("Should succeed in merging new ranges into empty DB data");

        let stored = get_stored_ranges(&db_guard, &region, &street);
        assert_eq!(stored.len(), 2, "Expected two disjoint ranges stored");
        assert_eq!(stored[0], range(10,15));
        assert_eq!(stored[1], range(20,25));
    }

    #[traced_test]
    fn test_existing_ranges_empty_new_ranges() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let (region, street) = test_region_and_street();
        let existing = vec![range(1, 5), range(10, 15)];
        let new = vec![]; // no new data

        put_existing_ranges(&mut db_guard, &region, &street, existing.clone());

        // Merging empty new data => existing data should remain unchanged
        integrate_house_number_subranges_for_street(&mut db_guard, &region, &street, &new)
            .expect("Merging empty new data should succeed");

        let stored = get_stored_ranges(&db_guard, &region, &street);
        assert_eq!(stored, existing, "Existing data should be unchanged");
    }

    #[traced_test]
    fn test_merge_overlapping_ranges() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let (region, street) = test_region_and_street();

        // Existing ranges: [10..=15], [30..=35]
        let existing = vec![range(10, 15), range(30, 35)];
        put_existing_ranges(&mut db_guard, &region, &street, existing);

        // New ranges overlap with [12..=18], plus disjoint [35..=40]
        let new = vec![range(12, 18), range(35, 40)];

        // After merge: 
        //  [10..=18], [30..=40]  but notice that [30..=35] is separate from [35..=40]
        //  They share a boundary at 35, so they unify => [30..=40].
        //  Final => [10..=18], [30..=40]
        integrate_house_number_subranges_for_street(&mut db_guard, &region, &street, &new)
            .expect("Should succeed in merging overlapping data");

        let stored = get_stored_ranges(&db_guard, &region, &street);
        assert_eq!(stored.len(), 2, "Should have 2 merged ranges total");
        assert_eq!(stored[0], range(10, 18));
        assert_eq!(stored[1], range(30, 40));
    }

    #[traced_test]
    fn test_merge_completely_overlapping_new_range() {
        let (db_arc, _td) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let (region, street) = test_region_and_street();
        // Existing: [100..=110], [120..=130]
        let existing = vec![range(100,110), range(120,130)];
        put_existing_ranges(&mut db_guard, &region, &street, existing);

        // New: single range [100..=130] which completely covers existing
        let new = vec![range(100,130)];

        integrate_house_number_subranges_for_street(&mut db_guard, &region, &street, &new)
            .expect("Should succeed in merging fully-overlapping data");

        let stored = get_stored_ranges(&db_guard, &region, &street);
        assert_eq!(stored.len(), 1, "Now all merges into one big range");
        assert_eq!(stored[0], range(100,130));
    }

    #[traced_test]
    fn test_load_error_logs_warning_but_succeeds() {
        // In real practice, you might implement a mocking layer for `load_existing_street_ranges`
        // to simulate an error. Since the code logs a warning and returns `None`,
        // we can proceed with the partial approach: we won't fully replicate that mock here,
        // but we can demonstrate that if a load fails, we proceed anyway.
        //
        // For demonstration, we'll define a custom DB type that returns an error
        // from load. This is just a quick approach, not a normal usage path.

        struct FailingLoadDatabase {
            inner: Database
        };
        impl FailingLoadDatabase {
            fn new(path: &Path) -> Self {
                let db = Database::open(path).expect("Could not open DB");
                Self { inner: db }
            }
        }
        impl LoadExistingStreetRanges for FailingLoadDatabase {
            fn load_existing_street_ranges(
                &self,
                _r: &WorldRegion,
                _s: &StreetName,
            ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
                // Always fail
                Err(DataAccessError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Simulated load error",
                )))
            }
        }
        // For the rest of the needed traits, delegate to self.inner or do minimal stubs:
        impl StoreHouseNumberRanges for FailingLoadDatabase {
            fn store_house_number_ranges(
                &mut self,
                _region: &WorldRegion,
                _street: &StreetName,
                _ranges: &[HouseNumberRange],
            ) -> Result<(), DatabaseConstructionError> {
                // We'll store in the real DB
                self.inner.store_house_number_ranges(_region, _street, _ranges)
            }
        }
        impl FailingLoadDatabase {
            fn integrate(&mut self, region: &WorldRegion, street: &StreetName, new_ranges: &Vec<HouseNumberRange>)
                -> Result<(), OsmPbfParseError>
            {
                integrate_house_number_subranges_for_street(self, region, street, new_ranges)
            }
            // Minimal stubs so the code compiles:
            fn lock(&self) -> Result<std::sync::MutexGuard<'_, Database>, ()> {
                panic!("Not used in failing load test");
            }
        }

        let temp_dir = TempDir::new().unwrap();
        let mut failing_db = FailingLoadDatabase::new(temp_dir.path());

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let street = StreetName::new("Example Street").unwrap();
        let new_ranges = vec![range(1,2)];

        // Should log a warning and still return Ok(())
        let result = failing_db.integrate(&region, &street, &new_ranges);
        assert!(result.is_ok(), "Should not be a hard error even if load fails");
    }

    #[traced_test]
    fn test_store_error_logs_warning_but_succeeds() {
        // Similarly, we can mock or override store in a minimal approach.

        struct FailingStoreDatabase {
            inner: Database
        };
        impl FailingStoreDatabase {
            fn new(path: &Path) -> Self {
                let db = Database::open(path).expect("Could not open DB");
                Self { inner: db.lock().unwrap().clone() }
            }
        }

        impl LoadExistingStreetRanges for FailingStoreDatabase {
            fn load_existing_street_ranges(
                &self,
                region: &WorldRegion,
                street: &StreetName
            ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
                // Real load
                self.inner.load_existing_street_ranges(region, street)
            }
        }

        impl StoreHouseNumberRanges for FailingStoreDatabase {
            fn store_house_number_ranges(
                &mut self,
                _region: &WorldRegion,
                _street: &StreetName,
                _ranges: &[HouseNumberRange],
            ) -> Result<(), DatabaseConstructionError> {
                // Always fail
                Err(DatabaseConstructionError::RocksDB(
                    rocksdb::Error::new("Simulated store failure")
                ))
            }
        }

        impl FailingStoreDatabase {
            fn integrate(&mut self, region: &WorldRegion, street: &StreetName, new_ranges: &Vec<HouseNumberRange>)
                -> Result<(), OsmPbfParseError>
            {
                integrate_house_number_subranges_for_street(self, region, street, new_ranges)
            }
        }

        let temp_dir = TempDir::new().unwrap();
        let mut failing_db = FailingStoreDatabase::new(temp_dir.path());

        // Put an existing range so we can confirm the code tries to merge
        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let street = StreetName::new("Fail Street").unwrap();
        // This store is a direct call on the "inner" DB, so it works
        failing_db.inner.store_house_number_ranges(&region, &street, &[range(10,15)])
            .unwrap();

        // Attempt to integrate
        let new_ranges = vec![range(12, 18)];
        let result = failing_db.integrate(&region, &street, &new_ranges);
        // Should log a warning but still return Ok
        assert!(result.is_ok(), "Store error logs a warning, not a hard error");
    }
}
