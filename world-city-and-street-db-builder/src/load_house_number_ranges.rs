// ---------------- [ File: src/load_house_number_ranges.rs ]
crate::ix!();

pub trait LoadHouseNumberRanges {
    fn load_house_number_ranges(
        &self, 
        region: &WorldRegion, 
        street_obj: &StreetName
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError>;
}

impl<I:StorageInterface> LoadHouseNumberRanges for DataAccess<I> {

    fn load_house_number_ranges(
        &self, 
        region: &WorldRegion, 
        street_obj: &StreetName
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> 
    {
        self.db().lock().expect("expected to be able to get the db").load_house_number_ranges(region,street_obj)
    }
}

impl LoadHouseNumberRanges for Database {

    // ----------------------------------------------------------------------
    // (C) Example method to load house-number ranges from DB or a stub
    // ----------------------------------------------------------------------
    //
    // This is purely illustrative. Adjust the signature or error handling 
    // as needed in your codebase.
    //
    fn load_house_number_ranges(
        &self, 
        region: &WorldRegion, 
        street_obj: &StreetName
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> 
    {
        let key = house_number_ranges_key(region, street_obj);
        let raw_opt = self.get(&key)?;

        match raw_opt {
            None => Ok(None), // no key => no data
            Some(bytes) => {
                // Attempt to decode from CBOR -> CompressedList<HouseNumberRange>
                let clist_result: Result<crate::compressed_list::CompressedList<HouseNumberRange>, _> =
                    serde_cbor::from_slice(&bytes);

                match clist_result {
                    Ok(clist) => {
                        let items = clist.items().clone();
                        Ok(Some(items))
                    }
                    Err(e) => {
                        let msg = format!(
                            "Failed to deserialize HouseNumberRanges for '{}': {}",
                            key, e
                        );
                        // Convert to OsmPbfParseError
                        Err(OsmPbfParseError::HouseNumberRangeSerdeError { msg }.into())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test_load_house_number_ranges {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    /// Creates a temporary database and returns `(Arc<Mutex<Database>>, TempDir)` so
    /// that the temp directory remains valid for the duration of the tests.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Creates a `DataAccess` that references the same `Database`.
    fn create_data_access<I:StorageInterface>(db: Arc<Mutex<I>>) -> DataAccess<I> {
        DataAccess::with_db(db)
    }

    /// Convenience for building a `HouseNumberRange`.
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// Helper to store an array of `HouseNumberRange` objects in the DB
    /// under the key that `load_house_number_ranges(...)` expects.
    /// This uses the DB's `store_house_number_ranges(...)`.
    fn store_house_number_ranges_for_test<I:StorageInterface>(
        db:     &mut I,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) {
        db.store_house_number_ranges(region, street, ranges)
          .expect("Storing house-number ranges should succeed in test setup");
    }

    #[traced_test]
    fn test_no_key_returns_none() {
        // If there's no key in the DB, we expect None as the result.
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("NoData Street").unwrap();

        let result = data_access
            .load_house_number_ranges(&region, &street)
            .expect("Should return Ok(None) if no data is stored");
        assert_eq!(result, None, "Expected None when no key is in DB");
    }

    #[traced_test]
    fn test_valid_data_returns_some_ranges() {
        // We'll store valid CBOR data (two house number ranges) and confirm we can load them.
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("ValidData St").unwrap();

        let ranges_in = vec![hnr(1, 10), hnr(20, 30)];
        store_house_number_ranges_for_test(&mut *db_guard, &region, &street, &ranges_in);

        // Release lock before reading to mimic real usage
        drop(db_guard);

        let loaded = data_access
            .load_house_number_ranges(&region, &street)
            .expect("Loading should succeed with valid data")
            .expect("We expect Some(...) with valid data stored");
        assert_eq!(loaded, ranges_in, "Loaded ranges should match stored data");
    }

    #[traced_test]
    fn test_corrupted_cbor_data_causes_error() {
        // We'll write invalid CBOR bytes, ensuring that an error is returned.
        let (db_arc, _tmp) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();
        let data_access = create_data_access(db_arc.clone());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Corrupted Street").unwrap();

        // Manually compute the key
        let key = house_number_ranges_key(&region, &street);
        db_guard.put(&key, b"not-valid-cbor").unwrap();

        // Release the lock to simulate normal read usage
        drop(db_guard);

        let result = data_access.load_house_number_ranges(&region, &street);

        match result {
            Err(DataAccessError::OsmPbfPareseError(e)) => {
                // The error message should indicate a deserialization failure
            }
            Err(e) => {
                panic!("Expected DataAccessError::OsmPbfParseError or decode-based error, but got: {:?}", e);
            }
            Ok(_) => panic!("Should not succeed with corrupted CBOR data"),
        }
    }

    #[traced_test]
    fn test_rocksdb_read_error_propagates() {
        // If a RocksDB read fails, that is typically converted to `DataAccessError::Io`.
        // We'll create a minimal stub that fails on `get(...)` to test the propagation.

        struct FailingDbStub;
        impl DatabaseGet for FailingDbStub {
            fn get(&self, _key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>, DataAccessError> {
                Err(DatabaseConstructionError::SimulatedReadError)
            }
        }

        // We only need `Database` for the trait, so let's do a partial structure:
        // We'll define enough to compile. The rest won't be called in this test.
        impl OpenDatabaseAtPath for FailingDbStub {
            fn open(_p: impl AsRef<std::path::Path>) -> Result<Arc<Mutex<Self>>, WorldCityAndStreetDbBuilderError> {
                unimplemented!()
            }
        }

        impl FailingDbStub {
            fn new() -> Self { Self }
        }

        // Provide the `load_house_number_ranges` for the stub
        impl LoadHouseNumberRanges for FailingDbStub {
            fn load_house_number_ranges(
                &self,
                _region: &WorldRegion,
                _street_obj: &StreetName
            ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
                let key = "HNR:fake";
                // We'll call get(...) => forced error
                match self.get(key) {
                    Ok(_) => Ok(None),
                    Err(e) => Err(DataAccessError::SimulatedReadError),
                }
            }
        }

        // Now let's create a test data access that uses this stub in place of a real DB
        #[derive(Clone)]
        struct FailingDataAccess {
            db: Arc<Mutex<FailingDbStub>>
        }
        impl FailingDataAccess {
            fn new(stub: FailingDbStub) -> Self {
                Self { db: Arc::new(Mutex::new(stub)) }
            }
        }
        impl LoadHouseNumberRanges for FailingDataAccess {
            fn load_house_number_ranges(
                &self,
                region: &WorldRegion,
                street_obj: &StreetName
            ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
                self.db.lock().unwrap().load_house_number_ranges(region, street_obj)
            }
        }

        // Build the failing data access
        let failing_access = FailingDataAccess::new(FailingDbStub::new());

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("Error Ave").unwrap();

        let result = failing_access.load_house_number_ranges(&region, &street);
        match result {
            Err(DataAccessError::SimulatedReadError) => { }
            other => {
                panic!("Expected SimulatedReadError error, got {:?}", other);
            }
        }
    }
}
