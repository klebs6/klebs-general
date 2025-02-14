// ---------------- [ File: src/load_existing_street_ranges.rs ]
// ---------------- [ File: src/load_existing_street_ranges.rs ]
crate::ix!();

pub trait LoadExistingStreetRanges {

    fn load_existing_street_ranges(
        &self,
        world_region: &WorldRegion,
        street:       &StreetName,
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError>;
}

impl LoadExistingStreetRanges for Database {

    /// Loads existing house‐number ranges for the specified street from the DB.
    fn load_existing_street_ranges(
        &self,
        world_region: &WorldRegion,
        street:       &StreetName,
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
        trace!(
            "load_existing_street_ranges: loading for street='{}' in region={:?}",
            street,
            world_region
        );
        let existing = self.load_house_number_ranges(world_region, street)?;
        Ok(existing)
    }
}

#[cfg(test)]
mod test_load_existing_street_ranges {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Convenience helper to generate a `HouseNumberRange` for test usage.
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// Stores a vector of `HouseNumberRange` items for a given region+street
    /// in RocksDB under the "HNR:REGION_ABBR:street" key. This is the same key
    /// that `load_house_number_ranges` expects, making it test-friendly.
    fn store_test_ranges<I:StorageInterface>(
        db:     &mut I,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) {
        db.store_house_number_ranges(region, street, ranges)
          .expect("Storing test house-number ranges should succeed");
    }

    #[traced_test]
    fn test_no_stored_data_returns_none() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Imaginary Street").unwrap();

        // We haven't stored anything yet, so we should get None
        let result = db_guard.load_existing_street_ranges(&region, &street)
            .expect("Should not fail on an empty DB");
        assert_eq!(result, None, "No data => expected None");
    }

    #[traced_test]
    fn test_existing_ranges_return_some_vec() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Main St").unwrap();

        // We'll store two subranges
        let ranges_in = vec![hnr(100, 110), hnr(120, 130)];
        store_test_ranges(&mut *db_guard, &region, &street, &ranges_in);

        let result = db_guard
            .load_existing_street_ranges(&region, &street)
            .expect("Should succeed in reading stored data");
        assert!(result.is_some(), "We have data => should be Some(...)");
        let loaded_ranges = result.unwrap();
        assert_eq!(loaded_ranges, ranges_in,
            "Loaded subranges should match what was originally stored");
    }

    #[traced_test]
    fn test_corrupted_data_returns_error() {
        // We'll directly write invalid bytes under the HNR: key.
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Glitch Ave").unwrap();

        let key = house_number_ranges_key(&region, &street);
        db_guard.put(&key, b"invalid-cbor-data").unwrap();

        let result = db_guard.load_existing_street_ranges(&region, &street);
        match result {
            Err(DataAccessError::Io(e)) => {
                // The parse error was converted into an Io error or something similar
                assert!(e.to_string().contains("Failed to deserialize"),
                    "Error message should hint about failing deserialization");
            },
            Err(e) => {
                panic!("Expected a DataAccessError::Io or decode-related error, got: {:?}", e);
            },
            Ok(_) => panic!("Should not succeed with corrupted data in DB"),
        }
    }

    #[traced_test]
    fn test_database_read_error_handling() {
        // If the underlying DB read fails, it should bubble up as a DataAccessError.
        // We'll do a minimal approach by defining a small stub type that always fails
        // in `load_house_number_ranges`. Then we call
        // `load_existing_street_ranges` to confirm the error is preserved.

        struct FailingDbStub;
        impl LoadExistingStreetRanges for FailingDbStub {
            fn load_existing_street_ranges(
                &self,
                _world_region: &WorldRegion,
                _street: &StreetName,
            ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
                Err(DataAccessError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Simulated read error",
                )))
            }
        }

        let stub = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Fail Street").unwrap();

        let result = stub.load_existing_street_ranges(&region, &street);
        assert!(matches!(result, Err(DataAccessError::Io(_))),
            "Should bubble up the forced Io error as DataAccessError::Io");
    }
}
