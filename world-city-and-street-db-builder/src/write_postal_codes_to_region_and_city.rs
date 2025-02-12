// ---------------- [ File: src/write_postal_codes_to_region_and_city.rs ]
// ---------------- [ File: src/write_postal_codes_to_region_and_city.rs ]
crate::ix!();

pub trait WritePostalCodesToRegionAndCity {
    fn write_postal_codes_to_region_and_city(
        &mut self, 
        region:       &WorldRegion, 
        city:         &CityName, 
        postal_codes: &BTreeSet<PostalCode>
    ) -> Result<(),DatabaseConstructionError>;
}

impl WritePostalCodesToRegionAndCity for Database {
    fn write_postal_codes_to_region_and_city(&mut self, region: &WorldRegion, city: &CityName, postal_codes: &BTreeSet<PostalCode>) 
        -> Result<(),DatabaseConstructionError> 
    {
        let key = c2z_key(region,city);
        self.put(&key, compress_set_to_cbor(postal_codes))?;
        Ok(())
    }
}

#[cfg(test)]
mod test_write_postal_codes_to_region_and_city {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// A small helper for building `PostalCode`.
    fn pc(country: Country, code: &str) -> PostalCode {
        PostalCode::new(country, code).unwrap()
    }

    /// Opens a temporary `Database` for testing.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
        (db, temp_dir)
    }

    /// Reads the stored postal codes for `(region, city)` by constructing `c2z_key`.
    /// Returns `None` if key is missing or decoding fails.
    fn load_postal_codes_from_c2z<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        city:   &CityName
    ) -> Option<BTreeSet<PostalCode>> {
        let key = c2z_key(region, city);
        match db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let clist_result: Result<crate::CompressedList<PostalCode>, _> =
                    serde_cbor::from_slice(&bytes);
                match clist_result {
                    Ok(clist) => Some(clist.items().iter().cloned().collect()),
                    Err(_) => None,
                }
            }
            _ => None,
        }
    }

    #[traced_test]
    fn test_write_nonempty_postalcodes_success() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("TestCity").unwrap();

        // We'll define two postal codes
        let mut pc_set = BTreeSet::new();
        pc_set.insert(pc(Country::USA, "21000"));
        pc_set.insert(pc(Country::USA, "21001"));

        db_guard
            .write_postal_codes_to_region_and_city(&region, &city, &pc_set)
            .expect("Should write successfully");

        // Now read back
        let loaded_opt = load_postal_codes_from_c2z(&db_guard, &region, &city);
        assert!(loaded_opt.is_some(), "Should have stored data");
        let loaded = loaded_opt.unwrap();
        assert_eq!(loaded, pc_set, "The stored data should match our input set");
    }

    #[traced_test]
    fn test_overwrite_existing_postal_codes() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let city = CityName::new("SomeCity").unwrap();

        // First store an initial set
        let mut initial_pc = BTreeSet::new();
        initial_pc.insert(pc(Country::USA, "22030"));
        db_guard
            .write_postal_codes_to_region_and_city(&region, &city, &initial_pc)
            .expect("write initial set");

        // Now store a new set
        let mut new_pc = BTreeSet::new();
        new_pc.insert(pc(Country::USA, "22031"));
        new_pc.insert(pc(Country::USA, "22032"));
        db_guard
            .write_postal_codes_to_region_and_city(&region, &city, &new_pc)
            .expect("write updated set");

        // Confirm old data replaced
        let stored_opt = load_postal_codes_from_c2z(&db_guard, &region, &city);
        let stored = stored_opt.unwrap();
        assert_eq!(stored, new_pc, "The new data should have overwritten the old data");
    }

    #[traced_test]
    fn test_write_empty_set() {
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let city = CityName::new("EmptyCity").unwrap();

        let empty_pc = BTreeSet::new();
        db_guard
            .write_postal_codes_to_region_and_city(&region, &city, &empty_pc)
            .expect("Should store empty set successfully");

        // Read back => should decode as empty set
        let loaded_opt = load_postal_codes_from_c2z(&db_guard, &region, &city);
        assert!(loaded_opt.is_some(), "Key should exist for empty set");
        let loaded = loaded_opt.unwrap();
        assert!(loaded.is_empty(), "Decoded set is empty");
    }

    #[traced_test]
    fn test_rocksdb_put_error() {
        // If put fails => returns DatabaseConstructionError::RocksDB
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
        // Combine with trait
        impl StorageInterface for FailingDbStub {}
        impl OpenDatabaseAtPath for FailingDbStub {
            fn open(_p: impl AsRef<std::path::Path>) 
                -> Result<Arc<Mutex<Self>>, DatabaseConstructionError> 
            {
                unimplemented!()
            }
        }
        impl WritePostalCodesToRegionAndCity for FailingDbStub {
            fn write_postal_codes_to_region_and_city(
                &mut self, 
                region: &WorldRegion, 
                city: &CityName, 
                postal_codes: &BTreeSet<PostalCode>
            ) -> Result<(),DatabaseConstructionError> {
                let key = c2z_key(region, city);
                let val = compress_set_to_cbor(postal_codes);
                self.put(key, val)?;
                Ok(())
            }
        }

        let mut db_stub = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("FailCity").unwrap();
        let mut pc_set = BTreeSet::new();
        pc_set.insert(pc(Country::USA, "99999"));

        let result = db_stub.write_postal_codes_to_region_and_city(&region, &city, &pc_set);
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated put error");
            }
            other => panic!("Expected RocksDB error, got {:?}", other),
        }
    }
}
