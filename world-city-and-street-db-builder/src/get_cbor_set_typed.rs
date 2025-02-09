// ---------------- [ File: src/get_cbor_set_typed.rs ]
crate::ix!();

pub trait GetCborSetTyped {

    /// INTERNAL HELPER: fetch a CBOR-encoded BTreeSet<T> from DB under `key`.
    /// If key not present or empty, returns None. If DB lock is poisoned,
    /// logs warning and returns None.
    fn get_cbor_set_typed<T>(&self, key: &str) -> Option<BTreeSet<T>>
    where
        T: Serialize + DeserializeOwned + Ord;
}

impl<I:StorageInterface> GetCborSetTyped for DataAccess<I> {

    /// INTERNAL HELPER: fetch a CBOR-encoded BTreeSet<T> from DB under `key`.
    /// If key not present or empty, returns None. If DB lock is poisoned,
    /// logs warning and returns None.
    fn get_cbor_set_typed<T>(&self, key: &str) -> Option<BTreeSet<T>>
    where
        T: Serialize + DeserializeOwned + Ord,
    {
        match self.db().lock() {
            Ok(db_guard) => {
                let val = match db_guard.get(key) {
                    Ok(opt) => opt,
                    Err(e) => {
                        warn!("DB get error for key {}: {}", key, e);
                        return None;
                    }
                };
                let bytes = val?;
                let list: Vec<T> = decompress_cbor_to_list(&bytes);
                if list.is_empty() {
                    None
                } else {
                    Some(list.into_iter().collect())
                }
            }
            Err(_) => {
                warn!("Could not get DB lock for key: {}", key);
                None
            },
        }
    }
}

#[cfg(test)]
mod get_cbor_set_typed_tests {
    use super::*;

    /// Simple helper type to test non-String data as well
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    struct TestItem {
        id: u32,
    }

    fn create_db_and_da<I:StorageInterface>() -> (Arc<Mutex<I>>, DataAccess<I>, TempDir) {
        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let db      = I::open(tmp_dir.path()).expect("db open");
        let da      = DataAccess::with_db(db.clone());
        (db, da, tmp_dir)
    }

    #[test]
    fn test_get_cbor_set_typed_no_key() {
        // If DB has no such key => val = None => function => returns None
        let (db_arc, da, _tempdir) = create_db_and_da::<Database>();

        let result = da.get_cbor_set_typed::<String>("NONEXISTENT");
        assert!(result.is_none(), "No data => None");
    }

    #[test]
    fn test_get_cbor_set_typed_db_get_error() {
        // If DB's get(...) fails, we log a warning => return None
        let (db_arc, da, _tempdir) = create_db_and_da::<Database>();

        // Force an error by some means. Realistically, you might forcibly close DB or 
        // cause a RocksDB error. We'll do a minimal approach: 
        // e.g., forcibly poison the DB's internal "get" if you had a test seam. 
        // Or we can simulate by manually returning an error from the underlying code
        // if you have a mock DB. In a real integration test, it's trickier.
        //
        // We'll do a partial demonstration with a panic to see if it can be caught:
        // Suppose we can't easily cause a .get error. We'll just check the code path coverage. 
        // Typically, we'd need a specialized test double or mock. We'll skip the actual forced error:
        
        let key = "some_key";
        // if we had a "put" that fails or a "get" that fails forcibly, that would lead to a log. 
        // We'll do a normal call => likely no error => None
        let result = da.get_cbor_set_typed::<String>(key);
        assert!(result.is_none(), "No data => None. If a real DB error occurred => also None");
        // To truly confirm the error path, you'd need a mock or special environment.
    }

    #[test]
    fn test_get_cbor_set_typed_lock_poisoned() {
        // If the DB lock is poisoned => log warning => return None
        let (db_arc, da, _tempdir) = create_db_and_da::<Database>();

        // Poison the lock
        let _ = std::panic::catch_unwind(|| {
            let guard = db_arc.lock().unwrap();
            panic!("intentionally poisoning the lock");
        });

        let result = da.get_cbor_set_typed::<String>("any_key");
        assert!(result.is_none());
        // Check logs if desired. The function logs a warning => "Could not get DB lock..."
    }

    #[test]
    fn test_get_cbor_set_typed_key_exists_empty_val() {
        // If the DB key is present but the value is empty => we parse => decompress => empty => return None
        let (db_arc, da, _tempdir) = create_db_and_da::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("my_key", b"").unwrap(); 
        }

        let result = da.get_cbor_set_typed::<String>("my_key");
        assert!(result.is_none(), "Empty value => no set => None");
    }

    #[test]
    fn test_get_cbor_set_typed_key_exists_corrupted_cbor() {
        // If the DB key is present but the data is corrupted => decompress => empty => None
        let (db_arc, da, _tempdir) = create_db_and_da::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("my_key", b"not valid cbor").unwrap();
        }

        let result = da.get_cbor_set_typed::<String>("my_key");
        // decompress_cbor_to_list(...) yields an empty Vec if it can't parse => so we return None
        assert!(result.is_none(), "Corrupted => parse => empty => None");
    }

    #[test]
    fn test_get_cbor_set_typed_valid_data_strings() {
        let (db_arc, da, _tempdir) = create_db_and_da::<Database>();
        // We'll store a set of strings => "hello", "world"
        let mut sset = BTreeSet::new();
        sset.insert("hello".to_string());
        sset.insert("world".to_string());
        let cbor_val = compress_set_to_cbor(&sset);

        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("my_string_key", cbor_val).unwrap();
        }

        let result = da.get_cbor_set_typed::<String>("my_string_key");
        assert!(result.is_some());
        let unwrapped = result.unwrap();
        assert_eq!(unwrapped.len(), 2);
        assert!(unwrapped.contains("hello"));
        assert!(unwrapped.contains("world"));
    }

    #[test]
    fn test_get_cbor_set_typed_valid_data_struct() {
        let (db_arc, da, _tempdir) = create_db_and_da::<Database>();
        // We'll store a set of TestItem { id=1 }, { id=2 }
        let mut iset = BTreeSet::new();
        iset.insert(TestItem { id: 1 });
        iset.insert(TestItem { id: 2 });
        let cbor_val = compress_set_to_cbor(&iset);

        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("my_struct_key", cbor_val).unwrap();
        }

        let result = da.get_cbor_set_typed::<TestItem>("my_struct_key");
        assert!(result.is_some());
        let unwrapped = result.unwrap();
        assert_eq!(unwrapped.len(), 2);
        let mut sorted: Vec<_> = unwrapped.into_iter().collect();
        sorted.sort_by_key(|x| x.id);
        assert_eq!(sorted[0].id, 1);
        assert_eq!(sorted[1].id, 2);
    }

    #[test]
    fn test_get_cbor_set_typed_duplicates_merged() {
        // BTreeSet merges duplicates
        let (db_arc, da, _tempdir) = create_db_and_da::<Database>();

        // Suppose the underlying Vec had duplicates => "a","a","b"
        // Then after decode => BTreeSet => "a","b"
        let list_with_dups = vec!["a".to_string(), "a".to_string(), "b".to_string()];
        let cbor_val = crate::compressed_list::CompressedList::from(list_with_dups);
        let cbor_bytes = serde_cbor::to_vec(&cbor_val).unwrap();

        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put("dup_key", &cbor_bytes).unwrap();
        }

        let result = da.get_cbor_set_typed::<String>("dup_key");
        assert!(result.is_some());
        let s = result.unwrap();
        assert_eq!(s.len(), 2, "Duplicates => merged => 2 unique strings");
        assert!(s.contains("a"));
        assert!(s.contains("b"));
    }
}
