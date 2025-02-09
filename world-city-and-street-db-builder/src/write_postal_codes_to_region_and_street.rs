// ---------------- [ File: src/write_postal_codes_to_region_and_street.rs ]
crate::ix!();

pub trait WritePostalCodesToRegionAndStreet {
    fn write_postal_codes_to_region_and_street(
        &mut self, 
        region:       &WorldRegion, 
        street:       &StreetName, 
        postal_codes: &BTreeSet<PostalCode>
    ) -> Result<(),DatabaseConstructionError>;
}

impl WritePostalCodesToRegionAndStreet for Database {
    fn write_postal_codes_to_region_and_street(&mut self, region: &WorldRegion, street: &StreetName, postal_codes: &BTreeSet<PostalCode>) 
        -> Result<(),DatabaseConstructionError> 
    {
        let key = s2z_key(region,street);
        self.put(&key, compress_set_to_cbor(postal_codes))?;
        Ok(())
    }
}

#[cfg(test)]
#[disable]
mod test_write_postal_codes_to_region_and_street {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// A small convenience for creating a `PostalCode`.
    fn pc(country: Country, code: &str) -> PostalCode {
        PostalCode::new(country, code).unwrap()
    }

    /// Opens a temporary `Database` for testing.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(tmp.path()).expect("Failed to open database in temp dir");
        (db, tmp)
    }

    /// Reads the stored postal codes for `(region, street)` by constructing the `s2z_key`.
    /// Returns `None` if key is missing or decoding fails.
    fn load_postal_codes_from_s2z<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        street: &StreetName
    ) -> Option<BTreeSet<PostalCode>> {
        let key = s2z_key(region, street);
        let val_opt = db.get(&key).ok()??;
        let clist_result: Result<crate::CompressedList<PostalCode>, _> =
            serde_cbor::from_slice(&val_opt);
        clist_result.ok().map(|cl| cl.items().clone().into_iter().collect())
    }

    #[test]
    fn test_write_nonempty_postalcodes_success() {
        let (db_arc, _temp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("TestStreet").unwrap();

        // We'll define two postal codes
        let mut pc_set = BTreeSet::new();
        pc_set.insert(pc(Country::USA, "21010"));
        pc_set.insert(pc(Country::USA, "21011"));

        db_guard
            .write_postal_codes_to_region_and_street(&region, &street, &pc_set)
            .expect("Should write successfully");

        // Now read back
        let loaded_opt = load_postal_codes_from_s2z(&db_guard, &region, &street);
        assert!(loaded_opt.is_some(), "Should have stored data");
        let loaded = loaded_opt.unwrap();
        assert_eq!(loaded, pc_set, "The stored data should match our input set");
    }

    #[test]
    fn test_overwrite_existing_postal_codes() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let street = StreetName::new("SomeStreet").unwrap();

        // First store an initial set
        let mut initial_pc = BTreeSet::new();
        initial_pc.insert(pc(Country::USA, "22040"));
        db_guard
            .write_postal_codes_to_region_and_street(&region, &street, &initial_pc)
            .expect("write initial set");

        // Now store a new set
        let mut new_pc = BTreeSet::new();
        new_pc.insert(pc(Country::USA, "22041"));
        new_pc.insert(pc(Country::USA, "22042"));
        db_guard
            .write_postal_codes_to_region_and_street(&region, &street, &new_pc)
            .expect("write updated set");

        // Confirm old data replaced
        let stored_opt = load_postal_codes_from_s2z(&db_guard, &region, &street);
        let stored = stored_opt.unwrap();
        assert_eq!(stored, new_pc, "The new data should have overwritten the old data");
    }

    #[test]
    fn test_write_empty_set() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let street = StreetName::new("EmptySt").unwrap();

        let empty_pc = BTreeSet::new();
        db_guard
            .write_postal_codes_to_region_and_street(&region, &street, &empty_pc)
            .expect("Should store empty set successfully");

        // Read back => should decode as empty set
        let loaded_opt = load_postal_codes_from_s2z(&db_guard, &region, &street);
        assert!(loaded_opt.is_some(), "Key should exist for empty set");
        let loaded = loaded_opt.unwrap();
        assert!(loaded.is_empty(), "Decoded set is empty");
    }

    #[test]
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
        impl WritePostalCodesToRegionAndStreet for FailingDbStub {
            fn write_postal_codes_to_region_and_street(
                &mut self, 
                region: &WorldRegion, 
                street: &StreetName, 
                postal_codes: &BTreeSet<PostalCode>
            ) -> Result<(),DatabaseConstructionError> {
                let key = s2z_key(region, street);
                let val = compress_set_to_cbor(postal_codes);
                self.put(key, val)?; // forcibly fails
                Ok(())
            }
        }

        let mut db_stub = FailingDbStub;
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let street = StreetName::new("FailStreet").unwrap();
        let mut pc_set = BTreeSet::new();
        pc_set.insert(pc(Country::USA, "99999"));

        let result = db_stub.write_postal_codes_to_region_and_street(&region, &street, &pc_set);
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated put error");
            }
            other => panic!("Expected DatabaseConstructionError::RocksDB, got {:?}", other),
        }
    }
}
