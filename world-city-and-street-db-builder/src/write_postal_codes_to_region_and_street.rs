// ---------------- [ File: src/write_postal_codes_to_region_and_street.rs ]
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
mod test_write_postal_codes_to_region_and_street {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// A small convenience for creating a `PostalCode`.
    fn pc(country: Country, code: &str) -> PostalCode {
        PostalCode::new(country, code).unwrap()
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

    #[traced_test]
    fn test_write_nonempty_postalcodes_success() {
        let (db_arc, _temp_dir) = create_temp_db::<Database>();
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
        let loaded_opt = load_postal_codes_from_s2z(&*db_guard, &region, &street);
        assert!(loaded_opt.is_some(), "Should have stored data");
        let loaded = loaded_opt.unwrap();
        assert_eq!(loaded, pc_set, "The stored data should match our input set");
    }

    #[traced_test]
    fn test_overwrite_existing_postal_codes() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
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
        let stored_opt = load_postal_codes_from_s2z(&*db_guard, &region, &street);
        let stored = stored_opt.unwrap();
        assert_eq!(stored, new_pc, "The new data should have overwritten the old data");
    }

    #[traced_test]
    fn test_write_empty_set() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let street = StreetName::new("EmptySt").unwrap();

        let empty_pc = BTreeSet::new();
        db_guard
            .write_postal_codes_to_region_and_street(&region, &street, &empty_pc)
            .expect("Should store empty set successfully");

        // Read back => should decode as empty set
        let loaded_opt = load_postal_codes_from_s2z(&*db_guard, &region, &street);
        assert!(loaded_opt.is_some(), "Key should exist for empty set");
        let loaded = loaded_opt.unwrap();
        assert!(loaded.is_empty(), "Decoded set is empty");
    }

    #[traced_test]
    fn test_rocksdb_put_error() {

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
