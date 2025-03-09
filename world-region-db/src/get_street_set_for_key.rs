// ---------------- [ File: src/get_street_set_for_key.rs ]
crate::ix!();

pub trait GetStreetSetForKey {

    /// Returns a set of StreetName objects for the given string key, if present.
    fn get_street_set(&self, key: &str) -> Option<BTreeSet<StreetName>>;
}

impl<I:StorageInterface> GetStreetSetForKey for DataAccess<I> {

    /// Returns a set of StreetName objects for the given string key, if present.
    fn get_street_set(&self, key: &str) -> Option<BTreeSet<StreetName>> {
        self.get_cbor_set_typed::<StreetName>(key)
    }
}

#[cfg(test)]
mod get_street_set_for_key_tests {
    use super::*;

    fn create_db_and_da<I:StorageInterface>() -> (Arc<Mutex<I>>, DataAccess<I>, TempDir) {
        let tmp = TempDir::new().unwrap();
        let db = I::open(tmp.path()).unwrap();
        let da = DataAccess::with_db(db.clone());
        (db, da, tmp)
    }

    #[traced_test]
    fn test_get_street_set_no_key() {
        let (_db_arc, da, _td) = create_db_and_da::<Database>();
        let result = da.get_street_set("nonexistent");
        assert!(result.is_none());
    }

    #[traced_test]
    fn test_get_street_set_empty_value() {
        let (db_arc, da, _td) = create_db_and_da::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("some_key", b"").unwrap();
        }
        let result = da.get_street_set("some_key");
        assert!(result.is_none());
    }

    #[traced_test]
    fn test_get_street_set_corrupt_data() {
        let (db_arc, da, _td) = create_db_and_da::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("bad_streets", b"not valid cbor").unwrap();
        }
        let result = da.get_street_set("bad_streets");
        assert!(result.is_none());
    }

    #[traced_test]
    fn test_get_street_set_valid_data() {
        let (db_arc, da, _td) = create_db_and_da::<Database>();
        let mut sset = BTreeSet::new();
        sset.insert(StreetName::new("Main St").unwrap());
        sset.insert(StreetName::new("Broadway").unwrap());
        let cbor = compress_set_to_cbor(&sset);

        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("streets_key", cbor).unwrap();
        }

        let result = da.get_street_set("streets_key");
        assert!(result.is_some());
        let unwrapped = result.unwrap();
        assert_eq!(unwrapped.len(), 2);
        let names: Vec<_> = unwrapped.iter().map(|sn| sn.name().to_string()).collect();
        // e.g. "main st", "broadway"
        assert!(names.contains(&"main st".to_string()));
        assert!(names.contains(&"broadway".to_string()));
    }

    #[traced_test]
    fn test_get_street_set_lock_poisoned() {
        let (db_arc, da, _td) = create_db_and_da::<Database>();
        // Poison
        let _ = std::panic::catch_unwind(|| {
            let guard = db_arc.lock().unwrap();
            panic!("poisoning lock");
        });
        // Then retrieval => None
        let result = da.get_street_set("whatever");
        assert!(result.is_none());
    }
}
