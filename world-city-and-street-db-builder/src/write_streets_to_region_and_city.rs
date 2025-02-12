// ---------------- [ File: src/write_streets_to_region_and_city.rs ]
// ---------------- [ File: src/write_streets_to_region_and_city.rs ]
crate::ix!();

pub trait WriteStreetsToRegionAndCity {
    fn write_streets_to_region_and_city(
        &mut self, 
        region:  &WorldRegion, 
        city:    &CityName, 
        streets: &BTreeSet<StreetName>
    ) -> Result<(), DatabaseConstructionError>;
}

impl WriteStreetsToRegionAndCity for Database {
    fn write_streets_to_region_and_city(&mut self, region: &WorldRegion, city: &CityName, streets: &BTreeSet<StreetName>) -> Result<(), DatabaseConstructionError> {
        let key = c2s_key(region,city);
        self.put(&key, compress_set_to_cbor(streets))?;
        Ok(())
    }
}

#[cfg(test)]
mod test_write_streets_to_region_and_city {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Helper to build a StreetName from a &str. Adjust if your constructor differs.
    fn street(name: &str) -> StreetName {
        StreetName::new(name).unwrap()
    }

    /// Opens a temporary `Database` and returns `(Arc<Mutex<Database>>, TempDir)`.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Reads back the stored streets for `(region, city)` by constructing `c2s_key`.
    /// Returns `None` if the key is missing or decoding fails.
    fn load_streets_from_c2s<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        city:   &CityName
    ) -> Option<BTreeSet<StreetName>> {
        let key = c2s_key(region, city);
        if let Ok(Some(bytes)) = db.get(key.as_bytes()) {
            let clist_result: Result<crate::CompressedList<StreetName>, _> =
                serde_cbor::from_slice(&bytes);
            if let Ok(clist) = clist_result {
                return Some(clist.items().iter().cloned().collect());
            }
        }
        None
    }

    #[traced_test]
    fn test_write_nonempty_streets_success() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("TestCity").unwrap();

        let mut streets = BTreeSet::new();
        streets.insert(street("MainStreet"));
        streets.insert(street("SecondAve"));

        // Store
        db_guard
            .write_streets_to_region_and_city(&region, &city, &streets)
            .expect("Should write successfully");

        // Read back
        let loaded_opt = load_streets_from_c2s(&db_guard, &region, &city);
        assert!(loaded_opt.is_some(), "Should have stored data");
        let loaded = loaded_opt.unwrap();
        assert_eq!(loaded, streets, "The stored data should match our input set");
    }

    #[traced_test]
    fn test_overwrite_existing_streets() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let city = CityName::new("SomeCity").unwrap();

        // First store an initial set
        let mut initial_streets = BTreeSet::new();
        initial_streets.insert(street("OldStreet"));
        db_guard
            .write_streets_to_region_and_city(&region, &city, &initial_streets)
            .expect("write initial set");

        // Now store a new set
        let mut new_streets = BTreeSet::new();
        new_streets.insert(street("NewStreet"));
        new_streets.insert(street("BrandNewLane"));
        db_guard
            .write_streets_to_region_and_city(&region, &city, &new_streets)
            .expect("write updated set");

        // Confirm old data replaced
        let stored_opt = load_streets_from_c2s(&db_guard, &region, &city);
        let stored = stored_opt.unwrap();
        assert_eq!(stored, new_streets, "The new data should have overwritten the old data");
    }

    #[traced_test]
    fn test_write_empty_set() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let city = CityName::new("EmptyCity").unwrap();

        let empty_streets = BTreeSet::new();
        db_guard
            .write_streets_to_region_and_city(&region, &city, &empty_streets)
            .expect("Should store empty set successfully");

        // Read back => should decode as empty set
        let loaded_opt = load_streets_from_c2s(&db_guard, &region, &city);
        assert!(loaded_opt.is_some(), "Key should exist for empty set");
        let stored = loaded_opt.unwrap();
        assert!(stored.is_empty(), "Decoded set is empty");
    }

    #[traced_test]
    fn test_rocksdb_put_error() {
        // If put fails => returns DatabaseConstructionError::RocksDB
        struct FailingDbStub;
        impl DatabasePut for FailingDbStub {
            fn put(
                &mut self, 
                _key: impl AsRef<[u8]>, 
                _val: impl AsRef<[u8]>
            ) -> Result<(), DatabaseConstructionError> {
                Err(DatabaseConstructionError::RocksDB(
                    rocksdb::Error::new("Simulated put error")
                ))
            }
        }
        // Combine with trait
        impl StorageInterface for FailingDbStub {}
        impl OpenDatabaseAtPath for FailingDbStub {
            fn open(_p: impl AsRef<std::path::Path>) 
                -> Result<Arc<Mutex<Self>>, DatabaseConstructionError> 
            {
                unimplemented!()
            }
        }
        impl WriteStreetsToRegionAndCity for FailingDbStub {
            fn write_streets_to_region_and_city(
                &mut self, 
                region: &WorldRegion, 
                city: &CityName, 
                streets: &BTreeSet<StreetName>
            ) -> Result<(), DatabaseConstructionError> {
                let key = c2s_key(region, city);
                let val = compress_set_to_cbor(streets);
                self.put(key, val)?; // forcibly fails
                Ok(())
            }
        }

        let mut db_stub = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("FailCity").unwrap();
        let mut st_set = BTreeSet::new();
        st_set.insert(street("FailStreet"));

        let result = db_stub.write_streets_to_region_and_city(&region, &city, &st_set);
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated put error");
            }
            other => panic!("Expected DatabaseConstructionError::RocksDB, got {:?}", other),
        }
    }
}
