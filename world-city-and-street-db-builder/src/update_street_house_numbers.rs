// ---------------- [ File: src/update_street_house_numbers.rs ]
crate::ix!();

/// Loads existing house‐number ranges for a street, merges new data, and stores the result.
pub fn update_street_house_numbers<I:StorageInterface>(
    db:         &mut I,
    region:     &WorldRegion,
    street:     &StreetName,
    new_ranges: &[HouseNumberRange],
) -> Result<(), DatabaseConstructionError> {

    trace!(
        "update_street_house_numbers: street='{}', merging {} new ranges",
        street,
        new_ranges.len()
    );

    let existing = db.load_existing_house_number_ranges(region, street)
        .map_err(|_| DatabaseConstructionError::DataAccessError)?;

    let merged = unify_new_and_existing_ranges(existing, new_ranges);

    db.store_house_number_ranges(region, street, &merged)?;

    Ok(())
}

#[cfg(test)]
#[disable]
mod test_update_street_house_numbers {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use std::collections::HashMap;

    /// A convenience for building a `HouseNumberRange`.
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// Opens a real or temp `Database`.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open database");
        (db, temp_dir)
    }

    /// Reads back the stored house number ranges to confirm the final result.
    /// Returns `None` if there's no stored data or if decoding fails.
    fn load_stored_ranges<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        street: &StreetName,
    ) -> Option<Vec<HouseNumberRange>> {
        match db.load_existing_house_number_ranges(region, street) {
            Ok(ranges) => Some(ranges),
            Err(_) => None, // If any error arises, we simplify to None
        }
    }

    #[test]
    fn test_no_existing_data_stores_new_ranges() {
        // (1) "no existing data" => new ranges stored directly
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Main St").unwrap();
        let new_ranges = vec![hnr(1,5), hnr(10,15)];

        // No data was stored => load_existing_house_number_ranges => returns an empty or nonexistent set
        update_street_house_numbers(&mut db_guard, &region, &street, &new_ranges)
            .expect("Should succeed with no existing data");

        // Now read back:
        let final_data = load_stored_ranges(&db_guard, &region, &street)
            .expect("Should find stored data");
        assert_eq!(final_data, new_ranges,
            "We expect the newly stored data to match what was provided, no merges needed");
    }

    #[test]
    fn test_existing_data_is_merged() {
        // (2) existing data => unify with new data => store merged
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("Merge Street").unwrap();

        // Suppose existing data: [10..20]
        // We'll store it directly, bypassing "update_street_house_numbers" for test setup:
        db_guard.store_house_number_ranges(&region, &street, &[hnr(10,20)])
            .expect("Initial store should succeed");

        // Now we call update_street_house_numbers with new = [15..25]
        let new_ranges = vec![hnr(15,25)];
        update_street_house_numbers(&mut db_guard, &region, &street, &new_ranges)
            .expect("Merging should succeed");

        // The result after unify: [10..25]
        let merged = load_stored_ranges(&db_guard, &region, &street)
            .expect("Should find stored data");
        assert_eq!(merged, vec![hnr(10,25)],
            "Overlapping subranges => unified => [10..25]");
    }

    #[test]
    fn test_existing_and_new_disjoint() {
        // If existing ranges disjoint from new => final = existing + new in sorted order
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let street = StreetName::new("Disjoint Ave").unwrap();

        // existing = [1..5, 20..30]
        let existing_data = vec![hnr(1,5), hnr(20,30)];
        db_guard.store_house_number_ranges(&region, &street, &existing_data)
            .expect("Should store initial ranges");

        // new = [10..12, 40..45]
        let new_ranges = vec![hnr(10,12), hnr(40,45)];
        update_street_house_numbers(&mut db_guard, &region, &street, &new_ranges)
            .expect("Disjoint merging should succeed");

        // final => [1..5, 10..12, 20..30, 40..45]
        let result = load_stored_ranges(&db_guard, &region, &street)
            .expect("Should read back merged data");
        let expected = vec![hnr(1,5), hnr(10,12), hnr(20,30), hnr(40,45)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_error_returns_data_access_error() {
        // We'll define a partial stub that fails on `load_existing_house_number_ranges`.
        // Then confirm update_street_house_numbers => DataAccessError.
        struct FailingLoadDb;
        impl LoadExistingHouseNumberRanges for FailingLoadDb {
            fn load_existing_house_number_ranges(
                &self, 
                _region: &WorldRegion, 
                _street: &StreetName
            ) -> Result<Vec<HouseNumberRange>, DataAccessError> {
                Err(DataAccessError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other, 
                    "Simulated load error"
                )))
            }
        }
        // We'll define enough for the trait calls:
        impl StoreHouseNumberRanges for FailingLoadDb {
            fn store_house_number_ranges(
                &mut self, 
                _region: &WorldRegion, 
                _street: &StreetName, 
                _ranges: &[HouseNumberRange]
            ) -> Result<(), DatabaseConstructionError> {
                panic!("Should not be called because loading fails first");
            }
        }
        // Merge with `Database` or a minimal struct that the code calls:
        impl StorageInterface for FailingLoadDb {}
        impl OpenDatabaseAtPath for FailingLoadDb {
            fn open(_path: impl AsRef<std::path::Path>) 
                -> Result<Arc<Mutex<Self>>, DatabaseConstructionError> 
            {
                unimplemented!()
            }
        }

        // We'll pass this failing DB into the function
        let mut db = FailingLoadDb;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Fail Street").unwrap();
        let new_ranges = vec![hnr(1,2)];

        let result = update_street_house_numbers(&mut db, &region, &street, &new_ranges);
        match result {
            Err(DatabaseConstructionError::DataAccessError) => {
                // Good
            }
            other => panic!("Expected DataAccessError, got {:?}", other),
        }
    }

    #[test]
    fn test_store_error_returns_error() {
        // If storing the final merged data fails => we see that error.
        struct FailingStoreDb {
            // We'll hold a vector for existing data, so we can pass the load step
            existing: Vec<HouseNumberRange>,
        }
        impl LoadExistingHouseNumberRanges for FailingStoreDb {
            fn load_existing_house_number_ranges(
                &self, 
                _region: &WorldRegion, 
                _street: &StreetName
            ) -> Result<Vec<HouseNumberRange>, DataAccessError> {
                Ok(self.existing.clone())
            }
        }
        impl StoreHouseNumberRanges for FailingStoreDb {
            fn store_house_number_ranges(
                &mut self,
                _region: &WorldRegion,
                _street: &StreetName,
                _ranges: &[HouseNumberRange],
            ) -> Result<(), DatabaseConstructionError> {
                Err(DatabaseConstructionError::RocksDB(
                    rocksdb::Error::new("Simulated store error")
                ))
            }
        }
        impl StorageInterface for FailingStoreDb {}
        impl OpenDatabaseAtPath for FailingStoreDb {
            fn open(_path: impl AsRef<std::path::Path>) -> Result<Arc<Mutex<Self>>, DatabaseConstructionError> {
                unimplemented!()
            }
        }

        let mut db = FailingStoreDb {
            existing: vec![hnr(10,20)],
        };
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("FailStreet").unwrap();
        let new_ranges = vec![hnr(15,25)]; // merging => [10..25]

        let result = update_street_house_numbers(&mut db, &region, &street, &new_ranges);
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated store error");
            }
            other => panic!("Expected DatabaseConstructionError::RocksDB, got {:?}", other),
        }
    }

    #[test]
    fn test_overlapping_merges_successfully() {
        // A simpler test verifying overlapping new data is merged with existing.
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("OverlapStreet").unwrap();

        // existing: [50..60]
        db_guard.store_house_number_ranges(&region, &street, &[hnr(50,60)])
            .expect("Initial store");

        // new: [55..65], unify => [50..65]
        let new_ranges = vec![hnr(55,65)];
        update_street_house_numbers(&mut db_guard, &region, &street, &new_ranges)
            .expect("Merging overlap should succeed");

        let final_data = load_stored_ranges(&db_guard, &region, &street)
            .expect("Should read final");
        let expected = vec![hnr(50,65)];
        assert_eq!(final_data, expected);
    }
}
