// ---------------- [ File: src/get_city_set_for_key.rs ]
// ---------------- [ File: src/get_city_set_for_key.rs ]
crate::ix!();

pub trait GetCitySetForKey {

    /// Returns a set of CityName objects for the given string key, if present.
    fn get_city_set(&self, key: &str) -> Option<BTreeSet<CityName>>;
}

impl<I:StorageInterface> GetCitySetForKey for DataAccess<I> {

    fn get_city_set(&self, key: &str) -> Option<BTreeSet<CityName>> {
        self.get_cbor_set_typed::<CityName>(key)
    }
}

#[cfg(test)]
mod get_city_set_for_key_tests {
    use super::*; // Ensure we can see `DataAccess, GetCitySetForKey, etc.`
    use crate::{
        CityName,     // or wherever CityName is declared
        compress_set_to_cbor,
    };
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Helper to create a test Database + DataAccess
    fn create_db_and_da<I:StorageInterface>() -> (Arc<Mutex<I>>, DataAccess<I>, TempDir) {
        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let db = I::open(tmp_dir.path()).expect("db open should succeed");
        let da = DataAccess::with_db(db.clone());
        (db, da, tmp_dir)
    }

    #[traced_test]
    fn test_get_city_set_no_such_key() {
        let (_db_arc, da, _tmp) = create_db_and_da::<Database>();
        let result = da.get_city_set("missing_key");
        assert!(result.is_none(), "No key => None");
    }

    #[traced_test]
    fn test_get_city_set_empty_value() {
        let (db_arc, da, _tmp) = create_db_and_da::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("some_key", b"").unwrap(); 
        }
        // If the stored value is empty => decompress => empty => returns None
        let result = da.get_city_set("some_key");
        assert!(result.is_none(), "Empty => None");
    }

    #[traced_test]
    fn test_get_city_set_corrupted_cbor() {
        let (db_arc, da, _tmp) = create_db_and_da::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("bad_key", b"not valid cbor").unwrap();
        }
        let result = da.get_city_set("bad_key");
        assert!(result.is_none(), "Corrupted => None");
    }

    #[traced_test]
    fn test_get_city_set_valid_data() {
        let (db_arc, da, _tmp) = create_db_and_da::<Database>();
        // We'll store e.g. {"baltimore","annapolis"} 
        // after normalization => "baltimore","annapolis"
        let mut cset = BTreeSet::new();
        cset.insert(CityName::new("Baltimore").unwrap());
        cset.insert(CityName::new("Annapolis").unwrap());
        let cbor = compress_set_to_cbor(&cset);

        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("cities_key", &cbor).unwrap();
        }

        let result = da.get_city_set("cities_key");
        assert!(result.is_some());
        let unwrapped = result.unwrap();
        // "baltimore" and "annapolis"
        assert_eq!(unwrapped.len(), 2);
        let mut names: Vec<String> = unwrapped
            .iter()
            .map(|cn| cn.name().to_owned())
            .collect();
        names.sort();
        assert_eq!(names, vec!["annapolis", "baltimore"]);
    }

    #[traced_test]
    fn test_get_city_set_lock_poisoned() {
        let (db_arc, da, _tmp) = create_db_and_da::<Database>();
        // Poison the lock:
        let _ = std::panic::catch_unwind(|| {
            let guard = db_arc.lock().unwrap();
            panic!("Forcing lock poison");
        });
        // Now retrieval => log warning => returns None
        let result = da.get_city_set("any_key");
        assert!(result.is_none());
    }
}
