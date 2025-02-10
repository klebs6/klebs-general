// ---------------- [ File: src/get_postal_code_set_for_key.rs ]
// ---------------- [ File: src/get_postal_code_set_for_key.rs ]
crate::ix!();

pub trait GetPostalCodeSetForKey {

    /// Returns a set of PostalCode objects for the given string key, if present.
    fn get_postal_code_set(&self, key: &str) -> Option<BTreeSet<PostalCode>>;
}

impl<I:StorageInterface> GetPostalCodeSetForKey for DataAccess<I> {

    /// Returns a set of PostalCode objects for the given string key, if present.
    fn get_postal_code_set(&self, key: &str) -> Option<BTreeSet<PostalCode>> {
        self.get_cbor_set_typed::<PostalCode>(key)
    }
}

#[cfg(test)]
mod get_postal_code_set_for_key_tests {
    use super::*;

    fn create_db_and_da<I:StorageInterface>() -> (Arc<Mutex<I>>, DataAccess<I>, TempDir) {
        let tmp_dir = TempDir::new().unwrap();
        let db      = I::open(tmp_dir.path()).unwrap();
        let da      = DataAccess::with_db(db.clone());
        (db, da, tmp_dir)
    }

    #[traced_test]
    fn test_get_postal_code_set_no_key() {
        let (_db_arc, da, _tmp) = create_db_and_da::<Database>();
        let result = da.get_postal_code_set("nonexistent");
        assert!(result.is_none());
    }

    #[traced_test]
    fn test_get_postal_code_set_empty_value() {
        let (db_arc, da, _tmp) = create_db_and_da::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("some_pc_key", b"").unwrap();
        }
        let result = da.get_postal_code_set("some_pc_key");
        assert!(result.is_none());
    }

    #[traced_test]
    fn test_get_postal_code_set_corrupted_cbor() {
        let (db_arc, da, _tmp) = create_db_and_da::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("bad_pc", b"not valid cbor").unwrap();
        }
        let result = da.get_postal_code_set("bad_pc");
        assert!(result.is_none());
    }

    #[traced_test]
    fn test_get_postal_code_set_valid_data() {
        let (db_arc, da, _tmp) = create_db_and_da::<Database>();
        // We'll store e.g. { "21201", "20850" }
        let mut pcset = BTreeSet::new();
        pcset.insert(PostalCode::new(Country::USA, "21201").unwrap());
        pcset.insert(PostalCode::new(Country::USA, "20850").unwrap());
        let cbor = compress_set_to_cbor(&pcset);

        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("pc_key", &cbor).unwrap();
        }

        let result = da.get_postal_code_set("pc_key");
        assert!(result.is_some());
        let unwrapped = result.unwrap();
        assert_eq!(unwrapped.len(), 2);
        // Check presence
        let codes: Vec<_> = unwrapped
            .iter()
            .map(|pc| pc.code().to_string())
            .collect();
        assert!(codes.contains(&"21201".to_string()));
        assert!(codes.contains(&"20850".to_string()));
    }

    #[traced_test]
    fn test_get_postal_code_set_lock_poisoned() {
        let (db_arc, da, _tmp) = create_db_and_da::<Database>();
        // Poison
        let _ = std::panic::catch_unwind(|| {
            let guard = db_arc.lock().unwrap();
            panic!("poison");
        });
        let result = da.get_postal_code_set("any");
        assert!(result.is_none());
    }
}
