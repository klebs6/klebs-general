// ---------------- [ File: src/store_merged_house_number_ranges.rs ]
crate::ix!();

/// Stores the merged list of house‐number ranges back into the database.
pub fn store_merged_house_number_ranges<I:StorageInterface>(
    db:     &mut I,
    region: &WorldRegion,
    street: &StreetName,
    merged: &[HouseNumberRange],
) -> Result<(), DatabaseConstructionError> {
    trace!(
        "store_merged_house_number_ranges: storing {} ranges for street='{}' in region={:?}",
        merged.len(),
        street,
        region
    );

    db.store_house_number_ranges(region, street, merged)?;
    debug!(
        "store_merged_house_number_ranges: successfully stored ranges for street='{}'",
        street
    );
    Ok(())
}

#[cfg(test)]
#[disable]
mod test_store_merged_house_number_ranges {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Creates a temporary database for testing, returning `(Arc<Mutex<Database>>, TempDir)`.
    /// The `TempDir` ensures the directory remains valid until the end of the test.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let tmp = TempDir::new().expect("Failed to create temp directory");
        let db = I::open(tmp.path()).expect("Failed to open database in temp dir");
        (db, tmp)
    }

    /// Convenience helper for constructing a `HouseNumberRange`.
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// Retrieves house-number ranges back from the DB to confirm correctness.
    /// Returns `None` if the key doesn't exist or decoding fails.
    fn load_ranges_from_db<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        street: &StreetName,
    ) -> Option<Vec<HouseNumberRange>> {
        // We rely on the same key used internally
        let key = house_number_ranges_key(region, street);
        let bytes_opt = db.get(key.as_bytes()).ok()??;
        let clist_result: Result<crate::compressed_list::CompressedList<HouseNumberRange>, _> =
            serde_cbor::from_slice(&bytes_opt);
        clist_result.ok().map(|cl| cl.items().clone())
    }

    #[test]
    fn test_store_merged_empty_ranges() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Empty Street").unwrap();

        let merged = vec![]; // no ranges
        let result = store_merged_house_number_ranges(&mut db_guard, &region, &street, &merged);
        assert!(result.is_ok(), "Storing empty ranges should succeed");

        let loaded = load_ranges_from_db(&db_guard, &region, &street)
            .expect("Should at least store an empty list, not None");
        assert!(loaded.is_empty(), "We wrote an empty set of ranges");
    }

    #[test]
    fn test_store_merged_some_ranges() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("Main St").unwrap();

        let merged = vec![hnr(1,10), hnr(20,30)];
        let result = store_merged_house_number_ranges(&mut db_guard, &region, &street, &merged);
        assert!(result.is_ok(), "Storing a non-empty list should succeed");

        let loaded = load_ranges_from_db(&db_guard, &region, &street)
            .expect("Should find stored data");
        assert_eq!(loaded, merged, "The data read back should match what was stored");
    }

    #[test]
    fn test_db_error_propagation() {
        // We can cause a DB error (e.g., RocksDB error on put) to ensure it returns an error.

        struct FailingDbStub;
        impl DatabasePut for FailingDbStub {
            fn put(&mut self, _key: impl AsRef<[u8]>, _val: impl AsRef<[u8]>) 
                -> Result<(), DatabaseConstructionError> 
            {
                Err(DatabaseConstructionError::RocksDB(
                    rocksdb::Error::new("Simulated put error")
                ))
            }
        }
        // For store_house_number_ranges
        impl StorageInterface for FailingDbStub {}
        impl OpenDatabaseAtPath for FailingDbStub {
            fn open(_p: impl AsRef<std::path::Path>) 
                -> Result<Arc<Mutex<Self>>, DatabaseConstructionError> 
            {
                unimplemented!()
            }
        }
        impl StoreHouseNumberRanges for FailingDbStub {
            fn store_house_number_ranges(
                &mut self, 
                _region: &WorldRegion,
                _street: &StreetName,
                _ranges: &[HouseNumberRange]
            ) -> Result<(), DatabaseConstructionError> {
                // We'll skip cbor logic for brevity in the stub
                // Just fail on put
                self.put(b"somekey", b"someval")?;
                Ok(())
            }
        }

        let mut failing_db = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Fail St").unwrap();
        let ranges = vec![hnr(5,10)];

        let result = store_merged_house_number_ranges(&mut failing_db, &region, &street, &ranges);
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated put error");
            }
            other => panic!("Expected RocksDB error, got {:?}", other),
        }
    }

    #[test]
    fn test_cbor_error_propagation() {
        // If the cbor serialization fails in store_house_number_ranges, we want to see 
        // the error returned from store_merged_house_number_ranges as well.

        struct FailingStreetName(StreetName);
        impl serde::Serialize for FailingStreetName {
            fn serialize<S>(
                &self, 
                _serializer: S
            ) -> Result<S::Ok, S::Error> 
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("Simulated cbor error on StreetName"))
            }
        }
        // We'll define a minimal approach: we can't trivially override HouseNumberRange
        // w/o rewriting. Instead, we can override store_house_number_ranges to force 
        // cbor error. We'll do a local function if needed:

        // For demonstration, we'll forcibly cause an error by rewriting store_house_number_ranges
        // in a minimal local function, or we define a partial stub that fails:

        struct CborFailingDb;
        impl DatabasePut for CborFailingDb {
            fn put(
                &mut self, 
                _key: impl AsRef<[u8]>, 
                _val: impl AsRef<[u8]>
            ) -> Result<(), DatabaseConstructionError> {
                Ok(())
            }
        }
        impl StorageInterface for CborFailingDb {}
        impl OpenDatabaseAtPath for CborFailingDb {
            fn open(_p: impl AsRef<std::path::Path>) 
                -> Result<Arc<Mutex<Self>>, DatabaseConstructionError> {
                unimplemented!()
            }
        }
        impl StoreHouseNumberRanges for CborFailingDb {
            fn store_house_number_ranges(
                &mut self,
                _region: &WorldRegion,
                _street: &StreetName,
                _ranges: &[HouseNumberRange],
            ) -> Result<(), DatabaseConstructionError> {
                // We'll forcibly cause cbor error
                let e = serde_cbor::Error::custom("Simulated cbor serialization error");
                let msg = format!("Failed to serialize HouseNumberRanges: {}", e);
                return Err(OsmPbfParseError::HouseNumberRangeSerdeError { msg }.into());
            }
        }

        let mut cbor_fail_db = CborFailingDb;
        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let street = StreetName::new("Some Street").unwrap();
        let ranges = vec![hnr(100,200)];

        let result = store_merged_house_number_ranges(&mut cbor_fail_db, &region, &street, &ranges);
        match result {
            Err(DatabaseConstructionError::OsmPbfParseError(
                OsmPbfParseError::HouseNumberRangeSerdeError{ msg }
            )) => {
                assert!(
                    msg.contains("Simulated cbor serialization error"),
                    "Should contain forced cbor error message, got: {}", msg
                );
            }
            other => panic!("Expected HouseNumberRangeSerdeError, got {:?}", other),
        }
    }
}
