// ---------------- [ File: src/write_streets_to_region_and_postal_code.rs ]
crate::ix!();

pub trait WriteStreetsToRegionAndPostalCode {
    fn write_streets_to_region_and_postal_code(
        &mut self, 
        region:      &WorldRegion, 
        postal_code: &PostalCode, 
        streets:     &BTreeSet<StreetName>
    ) -> Result<(),DatabaseConstructionError> ;
}

impl WriteStreetsToRegionAndPostalCode for Database {

    fn write_streets_to_region_and_postal_code(&mut self, region: &WorldRegion, postal_code: &PostalCode, streets: &BTreeSet<StreetName>) 
        -> Result<(),DatabaseConstructionError> 
    {
        let key = s_key(region,postal_code);
        let val = compress_set_to_cbor(streets);
        self.put(&key, val)?;
        Ok(())
    }
}

#[cfg(test)]
mod test_write_streets_to_region_and_postal_code {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// A small helper to create a `StreetName`. Adjust if your constructor differs.
    fn street(name: &str) -> StreetName {
        StreetName::new(name).unwrap()
    }

    /// Opens a temporary `Database` for testing.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Reads the stored streets for `(region, postal_code)` by constructing `s_key(region, postal_code)`.
    /// Returns `None` if the key is missing or decoding fails.
    fn load_streets_from_s_key<I:StorageInterface>(
        db:          &I,
        region:      &WorldRegion,
        postal_code: &PostalCode
    ) -> Option<BTreeSet<StreetName>> {
        let key = s_key(region, postal_code);
        let val_opt = db.get(&key).ok()??;
        let clist_result: Result<crate::CompressedList<StreetName>, _> =
            serde_cbor::from_slice(&val_opt);
        clist_result.ok().map(|cl| cl.items().iter().cloned().collect())
    }

    #[traced_test]
    fn test_write_nonempty_streets_success() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "21201").unwrap();

        let mut street_set = BTreeSet::new();
        street_set.insert(street("North Ave"));
        street_set.insert(street("Howard St"));

        // Write
        db_guard
            .write_streets_to_region_and_postal_code(&region, &postal_code, &street_set)
            .expect("Should write successfully");

        // Read back
        let loaded_opt = load_streets_from_s_key(&db_guard, &region, &postal_code);
        assert!(loaded_opt.is_some(), "Should have stored data");
        let loaded = loaded_opt.unwrap();
        assert_eq!(loaded, street_set, "The stored data should match our input set");
    }

    #[traced_test]
    fn test_overwrite_existing_streets() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let postal_code = PostalCode::new(Country::USA, "22030").unwrap();

        // First store some initial data
        let mut initial_streets = BTreeSet::new();
        initial_streets.insert(street("Old Rd"));
        db_guard
            .write_streets_to_region_and_postal_code(&region, &postal_code, &initial_streets)
            .expect("write initial set");

        // Now store a new set
        let mut new_streets = BTreeSet::new();
        new_streets.insert(street("New Blvd"));
        new_streets.insert(street("BrandNew Lane"));
        db_guard
            .write_streets_to_region_and_postal_code(&region, &postal_code, &new_streets)
            .expect("write updated set");

        // Confirm old data replaced
        let stored_opt = load_streets_from_s_key(&db_guard, &region, &postal_code);
        let stored = stored_opt.unwrap();
        assert_eq!(stored, new_streets, "The new data should have overwritten the old data");
    }

    #[traced_test]
    fn test_write_empty_set() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let postal_code = PostalCode::new(Country::USA, "20001").unwrap();

        let empty_streets = BTreeSet::new();
        db_guard
            .write_streets_to_region_and_postal_code(&region, &postal_code, &empty_streets)
            .expect("Should store empty set successfully");

        // Read back => should decode as empty set
        let loaded_opt = load_streets_from_s_key(&db_guard, &region, &postal_code);
        assert!(loaded_opt.is_some(), "Key should exist for empty set");
        let loaded = loaded_opt.unwrap();
        assert!(loaded.is_empty(), "Decoded set is empty");
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
        impl WriteStreetsToRegionAndPostalCode for FailingDbStub {
            fn write_streets_to_region_and_postal_code(
                &mut self, 
                region: &WorldRegion, 
                postal_code: &PostalCode, 
                streets: &BTreeSet<StreetName>
            ) -> Result<(),DatabaseConstructionError> {
                let key = s_key(region, postal_code);
                let val = compress_set_to_cbor(streets);
                self.put(&key, val)?; // forcibly fails
                Ok(())
            }
        }

        let mut db_stub = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "99999").unwrap();
        let mut st_set = BTreeSet::new();
        st_set.insert(street("FailStreet"));

        let result = db_stub.write_streets_to_region_and_postal_code(&region, &postal_code, &st_set);
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated put error");
            }
            other => panic!("Expected DatabaseConstructionError::RocksDB, got {:?}", other),
        }
    }
}
