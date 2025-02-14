// ---------------- [ File: src/store_house_number_ranges.rs ]
crate::ix!();

pub trait StoreHouseNumberRanges {
    fn store_house_number_ranges(
        &mut self,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) -> Result<(), DatabaseConstructionError>;
}

impl StoreHouseNumberRanges for Database {

    /// Stores a set of house number sub-ranges for a given region/street into RocksDB.
    ///
    /// This overwrites any existing data for that region+street. 
    /// If you want to merge or append, load first, modify, then store again.
    fn store_house_number_ranges(
        &mut self,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) -> Result<(), DatabaseConstructionError> {
        // 1) Key = "HNR:REGION_ABBR:street"
        let key = house_number_ranges_key(region, street);

        // 2) We'll store in CBOR. We can store it as a vector of HouseNumberRange 
        //    inside the standard CompressedList container or just directly. 
        //    For consistency with the rest of the code, let's store it in a CompressedList.
        let clist = crate::compressed_list::CompressedList::from(ranges.to_vec());
        let serialized = match serde_cbor::to_vec(&clist) {
            Ok(bytes) => bytes,
            Err(e) => {
                // Convert to OsmPbfParseError
                let msg = format!("Failed to serialize HouseNumberRanges for street '{}': {}", street.name(), e);
                return Err(OsmPbfParseError::HouseNumberRangeSerdeError { msg }.into());
            }
        };

        // 3) Put into RocksDB
        self.put(key.as_bytes(), serialized)?;
        Ok(())
    }
}

#[cfg(test)]
mod test_store_house_number_ranges {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// A convenience for creating a [`HouseNumberRange`].
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// Retrieves the stored data from the DB by the same key and attempts to decode
    /// it back into a `Vec<HouseNumberRange>`. If none or fail => returns `None`.
    fn read_stored_ranges<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        street: &StreetName
    ) -> Option<Vec<HouseNumberRange>> {
        let key = house_number_ranges_key(region, street);
        let bytes_opt = db.get(key.as_bytes()).ok()??;
        let clist_result: Result<crate::compressed_list::CompressedList<HouseNumberRange>, _> =
            serde_cbor::from_slice(&bytes_opt);
        clist_result.ok().map(|cl| cl.items().clone())
    }

    #[traced_test]
    fn test_store_empty_range_list() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Empty Street").unwrap();
        let empty_ranges = vec![];

        // Store empty
        db_guard
            .store_house_number_ranges(&region, &street, &empty_ranges)
            .expect("Storing empty range list should succeed");

        // Retrieve
        let loaded_opt = read_stored_ranges(&*db_guard, &region, &street);
        assert!(loaded_opt.is_some(), "Should have stored an empty list");
        let loaded = loaded_opt.unwrap();
        assert!(loaded.is_empty(), "Expected an empty list after storing empty ranges");
    }

    #[traced_test]
    fn test_store_non_empty_ranges() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("Main St").unwrap();
        let ranges_in = vec![hnr(1, 5), hnr(10, 15)];

        // Store
        db_guard
            .store_house_number_ranges(&region, &street, &ranges_in)
            .expect("Should succeed storing valid range list");

        // Retrieve
        let loaded_opt = read_stored_ranges(&*db_guard, &region, &street);
        assert!(loaded_opt.is_some(), "Should find a stored list");
        let loaded = loaded_opt.unwrap();
        assert_eq!(loaded, ranges_in, "Loaded data should match what was stored");
    }

    #[traced_test]
    fn test_overwrites_existing_data() {
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let street = StreetName::new("Pennsylvania Ave").unwrap();

        // First store some initial data
        let initial_data = vec![hnr(1, 10), hnr(20, 30)];
        db_guard
            .store_house_number_ranges(&region, &street, &initial_data)
            .expect("Should store initial data");

        // Now overwrite with new data
        let new_data = vec![hnr(100, 200)];
        db_guard
            .store_house_number_ranges(&region, &street, &new_data)
            .expect("Should store new data and overwrite old");

        // Confirm old data is gone, replaced by new
        let loaded_opt = read_stored_ranges(&*db_guard, &region, &street);
        let loaded = loaded_opt.unwrap();
        assert_eq!(loaded.len(), 1, "Should only have new data now");
        assert_eq!(loaded[0], hnr(100,200));
    }

    #[traced_test]
    fn test_serde_error_returns_error() {
        // We want to simulate a failure in `serde_cbor::to_vec(...)`. 
        // However, `HouseNumberRange` is trivially serializable. We'll do a minimal approach:
        //   - We define a mock or we forcibly cause an error by building an invalid structure.
        // 
        // It's actually quite hard to make serde_cbor fail with a normal struct. 
        // We'll define a minimal approach: a "FailingRange" that fails on serialization 
        // and a local function that calls the real store logic but casts our type. 
        // Or we can temporarily replace HouseNumberRange with an un-serializable type. 
        //
        // For demonstration, let's define a local trait implement that fails, then do 
        // partial patching or mocking. We'll do a simpler approach: we won't even call 
        // `serde_cbor::to_vec(...)` with real data, but define a local function that
        // is identical to store_house_number_ranges but we forcibly cause an error.

        struct FailingRange;
        impl serde::Serialize for FailingRange {
            fn serialize<S>(
                &self, 
                _serializer: S
            ) -> Result<S::Ok, S::Error> 
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("Simulated serialization failure"))
            }
        }

        fn store_failing_range<I:StorageInterface>(
            db:      &mut I,
            region:  &WorldRegion,
            street:  &StreetName,
            _ranges: &[FailingRange],
        ) -> Result<(), DatabaseConstructionError> {
            let key = house_number_ranges_key(region, street);

            // Force failure
            let failing_data = serde_cbor::to_vec(&_ranges)
                .map_err(|e| {
                    let msg = format!("Failed to serialize HouseNumberRanges for street '{}': {}", street.name(), e);
                    OsmPbfParseError::HouseNumberRangeSerdeError { msg }
                })?;
            
            db.put(key.as_bytes(), failing_data)?;
            Ok(())
        }

        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Fail Street").unwrap();

        let failing_ranges = vec![FailingRange];

        let result = store_failing_range(&mut *db_guard, &region, &street, &failing_ranges);
        match result {
            Err(DatabaseConstructionError::OsmPbfParseError(
                OsmPbfParseError::HouseNumberRangeSerdeError { msg }
            )) => {
                assert!(
                    msg.contains("Simulated serialization failure"),
                    "Expected forced error in message. Got: {}", msg
                );
            }
            other => panic!("Expected OsmPbfParseError::HouseNumberRangeSerdeError, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_rocksdb_error_in_db_put() {

        let mut db_stub = FailingDbStub;
        let region      = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street      = StreetName::new("Rocky Road").unwrap();
        let ranges_in   = vec![hnr(1, 2)];

        let result = db_stub.store_house_number_ranges(&region, &street, &ranges_in);

        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated RocksDB put error");
            }
            other => panic!("Expected DatabaseConstructionError::RocksDB, got {:?}", other),
        }
    }
}
