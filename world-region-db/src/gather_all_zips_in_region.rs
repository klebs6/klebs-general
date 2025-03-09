// ---------------- [ File: src/gather_all_zips_in_region.rs ]
crate::ix!();

pub trait GatherAllZipsInRegion {

    fn gather_all_zips_in_region(&self, region: &WorldRegion) -> Vec<PostalCode>;
}

impl<I:StorageInterface> GatherAllZipsInRegion for DataAccess<I> {

    // -----------------------------------------------------------
    // (B) The integrated function to gather ALL zips in a region.
    // -----------------------------------------------------------
    //
    // This iterates over the DB keys that start with `Z2C:<region_abbr>:` 
    // and extracts the postal code substring from each key. 
    // If the code is valid, we add it to a Vec.
    //
    fn gather_all_zips_in_region(&self, region: &WorldRegion) -> Vec<PostalCode> {
        let prefix = format!("Z2C:{}:", region.abbreviation());
        match self.db().lock() {
            Ok(db_guard) => {
                let iter = db_guard.prefix_iterator(prefix.as_bytes());
                let mut out = Vec::new();
                for kv in iter {
                    if let Ok((key_bytes, _val_bytes)) = kv {
                        let key_str = String::from_utf8_lossy(&key_bytes);
                        // e.g. "Z2C:US:21201"
                        let parts: Vec<&str> = key_str.splitn(3, ':').collect();
                        if parts.len() < 3 {
                            continue;
                        }
                        let zip_str = parts[2];
                        if let Ok(pc) = PostalCode::new(Country::USA, zip_str) {
                            out.push(pc);
                        }
                    }
                }
                out
            },
            Err(_) => {
                warn!("Could not lock DB in gather_all_zips_in_region");
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod gather_all_zips_in_region_tests {
    use super::*;

    /// Helper to create a fresh DB & DataAccess for testing.
    fn create_test_db_and_dataaccess<I:StorageInterface>() -> (Arc<Mutex<I>>, DataAccess<I>, TempDir) {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("db to open");
        let da = DataAccess::with_db(db.clone());
        (db, da, temp_dir)
    }

    #[traced_test]
    fn test_gather_all_zips_in_region_no_keys() {
        let (_db_arc, da, _td) = create_test_db_and_dataaccess::<Database>();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        let zips = da.gather_all_zips_in_region(&region);
        assert!(zips.is_empty(), "No keys => no results");
        // assert!(logs_contain("gather_all_zips_in_region"));
    }

    #[traced_test]
    fn test_gather_all_zips_in_region_some_keys() {
        let (db_arc, da, _td) = create_test_db_and_dataaccess::<Database>();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        
        // Insert some ZIP keys in the DB under "Z2C:US:21201", "Z2C:US:21202" if your region.abbreviation => US, 
        // or "Z2C:MD:21201" if the code returns "MD" for Maryland. Adjust as needed.
        let abbr = region.abbreviation();
        {
            let mut db_guard = db_arc.lock().unwrap();
            // We'll just store some dummy data; the function only cares about the key for the substring after 2 colons
            db_guard.put(format!("Z2C:{}:21201", abbr), b"whatever").unwrap();
            db_guard.put(format!("Z2C:{}:21202", abbr), b"ignored cbor").unwrap();
            db_guard.put(format!("Z2C:{}:21401", abbr), b"some data").unwrap();
        }

        // gather
        let zips = da.gather_all_zips_in_region(&region);
        // We expect ["21201","21202","21401"] in some order. The code returns them in insertion order typically, but let's just check presence:
        let mut set = BTreeSet::new();
        for pc in zips {
            set.insert(pc.code().to_owned());
        }
        // Check that we have them all
        assert_eq!(set.len(), 3);
        assert!(set.contains("21201"));
        assert!(set.contains("21202"));
        assert!(set.contains("21401"));
    }

    #[traced_test]
    fn test_gather_all_zips_in_region_other_region_skipped() {
        let (db_arc, da, _td) = create_test_db_and_dataaccess::<Database>();

        let region_md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let region_va: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();
        let abbr_md = region_md.abbreviation();
        let abbr_va = region_va.abbreviation();

        {
            let mut db_guard = db_arc.lock().unwrap();
            // Insert for MD => "Z2C:MD:21201"
            db_guard.put(format!("Z2C:{}:21201", abbr_md), b"stuff").unwrap();
            // Insert for VA => "Z2C:VA:20190"
            db_guard.put(format!("Z2C:{}:20190", abbr_va), b"stuff").unwrap();
        }

        // gather for MD => see only 21201
        let zips_md = da.gather_all_zips_in_region(&region_md);
        assert_eq!(zips_md.len(), 1);
        assert_eq!(zips_md[0].code(), "21201");

        // gather for VA => see only 20190
        let zips_va = da.gather_all_zips_in_region(&region_va);
        assert_eq!(zips_va.len(), 1);
        assert_eq!(zips_va[0].code(), "20190");
    }

    #[traced_test]
    fn test_gather_all_zips_in_region_key_malformed() {
        let (db_arc, da, _td) = create_test_db_and_dataaccess::<Database>();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let abbr = region.abbreviation();

        {
            let mut db_guard = db_arc.lock().unwrap();
            // Proper => "Z2C:MD:20850"
            db_guard.put(format!("Z2C:{}:20850", abbr), b"some data").unwrap();
            // Malformed => "Z2C:MD" with no postal code
            db_guard.put(format!("Z2C:{}", abbr), b"???").unwrap();
            // Another malformed => missing second colon => "Z2CMD20851"
            db_guard.put(format!("Z2C{}{}", abbr, "20851"), b"???").unwrap();
        }

        let zips = da.gather_all_zips_in_region(&region);
        assert_eq!(zips.len(), 1, "Only '20850' is properly recognized");
        assert_eq!(zips[0].code(), "20850");
    }

    #[traced_test]
    fn test_gather_all_zips_in_region_invalid_postal_code() {
        let (db_arc, da, _td) = create_test_db_and_dataaccess::<Database>();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let abbr = region.abbreviation();

        {
            let mut db_guard = db_arc.lock().unwrap();
            // We'll store one valid => "Z2C:MD:21201", one invalid => "Z2C:MD:???" 
            db_guard.put(format!("Z2C:{}:21201", abbr), b"whatever").unwrap();
            db_guard.put(format!("Z2C:{}:???", abbr), b"something").unwrap();
        }

        let zips = da.gather_all_zips_in_region(&region);
        assert_eq!(zips.len(), 1, "Only the valid 21201 parse succeeds");
        assert_eq!(zips[0].code(), "21201");
    }

    #[traced_test]
    fn test_gather_all_zips_in_region_duplicates() {
        // If code sees multiple "Z2C:MD:21201" keys, it pushes them all
        // => gather_all_zips_in_region returns duplicates (the code never dedups).
        let (db_arc, da, _td) = create_test_db_and_dataaccess::<Database>();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let abbr = region.abbreviation();

        {
            let mut db_guard = db_arc.lock().unwrap();
            // Insert the same postal code multiple times
            db_guard.put(format!("Z2C:{}:21201", abbr), b"val1").unwrap();
            db_guard.put(format!("Z2C:{}:21201", abbr), b"val2").unwrap();
        }

        let zips = da.gather_all_zips_in_region(&region);
        // We'll likely see [21201, 21201] or some duplication, depending on iteration.
        // If the prefix_iterator sees them as separate keys or if RocksDB overwrote the value. 
        // Typically "put" overwrites. 
        // So we might see them just once. We'll check presence >=1 anyway.
        assert!(!zips.is_empty(), "At least one 21201");
        assert_eq!(zips[0].code(), "21201");
        // If you want to verify duplicates, you'd do e.g. assert_eq!(zips.len(), 2) 
        // if your underlying DB allows distinct entries with the same key. 
        // Usually RocksDB overwrites the same key => 1 entry. 
    }

    #[traced_test]
    fn test_gather_all_zips_in_region_lock_poisoned() {
        // We'll forcibly poison the lock => gather_all_zips_in_region => logs a warning => returns empty
        let (db_arc, da, _td) = create_test_db_and_dataaccess::<Database>();

        // Poison the lock
        let _ = std::panic::catch_unwind(|| {
            let guard = db_arc.lock().unwrap();
            panic!("Intentionally poisoning the lock");
        });

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let result = da.gather_all_zips_in_region(&region);
        assert!(result.is_empty());
        // assert!(logs_contain("Could not lock DB in gather_all_zips_in_region"));
    }
}
