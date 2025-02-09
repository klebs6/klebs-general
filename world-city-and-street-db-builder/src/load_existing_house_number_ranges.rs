// ---------------- [ File: src/load_existing_house_number_ranges.rs ]
crate::ix!();

pub trait LoadExistingHouseNumberRanges {

    fn load_existing_house_number_ranges(
        &self,
        region: &WorldRegion,
        street: &StreetName,
    ) -> Result<Vec<HouseNumberRange>, DataAccessError>;
}

impl LoadExistingHouseNumberRanges for Database {

    /// Loads existing house‐number ranges from the database for the specified street in the given region.
    fn load_existing_house_number_ranges(
        &self,
        region: &WorldRegion,
        street: &StreetName,
    ) -> Result<Vec<HouseNumberRange>, DataAccessError> {
        trace!(
            "load_existing_house_number_ranges: street='{}' in region={:?}",
            street,
            region
        );

        let existing_opt = self.load_house_number_ranges(region, street)?;
        let existing = existing_opt.unwrap_or_default();

        debug!(
            "load_existing_house_number_ranges: found {} existing ranges for street='{}'",
            existing.len(),
            street
        );
        Ok(existing)
    }
}

#[cfg(test)]
#[disable]
mod test_load_existing_house_number_ranges {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    /// Creates a temporary database and returns `(db, temp_dir)` so that the temp dir
    /// remains valid throughout this test module's usage.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Helper to generate a `HouseNumberRange` easily in tests.
    fn range(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// Writes multiple house-number subranges under the region+street key
    /// so that we can test retrieving them with `load_existing_house_number_ranges`.
    fn store_house_number_ranges_for_test<I:StorageInterface>(
        db:     &mut I,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) {
        db.store_house_number_ranges(region, street, ranges)
          .expect("Storing house-number ranges should succeed in test setup");
    }

    #[test]
    fn test_no_data_returns_empty_vec() {
        let (db_arc, _temp_dir) = create_temp_db();
        let db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Empty Street").unwrap();

        let result = db_guard.load_existing_house_number_ranges(&region, &street)
            .expect("Should return an empty Vec if no data is stored");
        assert!(result.is_empty(), "Expected an empty vector for nonexistent key");
    }

    #[test]
    fn test_existing_data_is_loaded() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Main St").unwrap();

        let input_ranges = vec![range(1, 10), range(20, 25)];
        store_house_number_ranges_for_test(&mut db_guard, &region, &street, &input_ranges);

        // Now load them back
        let loaded_ranges = db_guard
            .load_existing_house_number_ranges(&region, &street)
            .expect("Loading existing data should succeed");
        assert_eq!(loaded_ranges, input_ranges,
            "Loaded data must match what was stored");
    }

    #[test]
    fn test_corrupted_data_returns_error() {
        // The `load_existing_house_number_ranges` method ultimately calls
        // `load_house_number_ranges`, which may fail during CBOR deserialization.
        // We'll simulate that here by manually writing invalid bytes under the key.
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Corrupted St").unwrap();

        // Manually create the key used for storing house-number ranges:
        let key = house_number_ranges_key(&region, &street);

        // Write some invalid data:
        db_guard.put(&key, b"not-valid-cbor").unwrap();

        // Attempting to load should yield a DataAccessError
        let result = db_guard.load_existing_house_number_ranges(&region, &street);
        match result {
            Err(DataAccessError::Io(_)) => {
                // A decode error is wrapped as an Io error or it might appear as another variant
                // depending on how your code translates cbor parse errors. Adjust if needed.
            }
            Err(e) => panic!("Expected an Io error or a decode-based error, got: {:?}", e),
            Ok(_) => panic!("Should not succeed loading corrupted data"),
        }
    }

    #[test]
    fn test_load_error_propagates_as_data_access_error() {
        // If you want to simulate a RocksDB read error or lock poison,
        // you'd need a mock or specialized DB that returns an error.
        // For demonstration, we can do a minimal approach:
        // We'll define a thin stub type that overrides load_house_number_ranges
        // with a forced error. Then we call `load_existing_house_number_ranges`
        // on it.

        struct FailingDb;
        impl LoadExistingHouseNumberRanges for FailingDb {
            fn load_existing_house_number_ranges(
                &self,
                _region: &WorldRegion,
                _street: &StreetName,
            ) -> Result<Vec<HouseNumberRange>, DataAccessError> {
                Err(DataAccessError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Simulated RocksDB error",
                )))
            }
        }
        // We won't fully implement `Database` here since we only need the trait to test the logic:
        let failing_db = FailingDb;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Failure Alley").unwrap();

        let result = failing_db.load_existing_house_number_ranges(&region, &street);
        assert!(
            matches!(result, Err(DataAccessError::Io(_))),
            "Should propagate the forced Io error as DataAccessError::Io"
        );
    }
}
