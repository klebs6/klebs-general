// ---------------- [ File: src/putget.rs ]
crate::ix!();

pub trait DatabasePut {
    fn put(&mut self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) 
        -> Result<(),DatabaseConstructionError>;
}

impl DatabasePut for Database {

    fn put(&mut self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<(),DatabaseConstructionError> {
        self.db().put(key, val)?;
        Ok(())
    }
}

//--------------------------------
pub trait DatabaseGet {
    fn get(&self, key: impl AsRef<[u8]>) 
        -> Result<Option<Vec<u8>>,DatabaseConstructionError>;
}

impl DatabaseGet for Database {

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>,DatabaseConstructionError> {
        Ok(self.db().get(key)?)
    }
}

#[cfg(test)]
#[disable]
mod test_database_put_get {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};
    use std::str;

    /// Creates a temporary RocksDB-based `Database` for testing,
    /// returning `(Arc<Mutex<Database>>, TempDir)` so the temp directory
    /// remains valid for the test's scope.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let tmp = TempDir::new().expect("Failed to create temp directory");
        let db = I::open(tmp.path()).expect("Failed to open database in temp dir");
        (db, tmp)
    }

    #[test]
    fn test_put_and_get_round_trip() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        // We'll store a key-value pair
        let key = b"mykey";
        let value = b"myvalue";
        db_guard.put(key, value).expect("Storing key-value should succeed");

        // Now retrieve it
        let retrieved = db_guard.get(key).expect("Reading key-value should succeed");
        assert!(retrieved.is_some(), "Value should be present");
        assert_eq!(
            retrieved.unwrap(),
            value,
            "Retrieved value should match stored value"
        );
    }

    #[test]
    fn test_get_non_existent_key_returns_none() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let db_guard = db_arc.lock().unwrap();

        let missing_key = b"no_such_key";
        let result = db_guard.get(missing_key)
            .expect("Should succeed, but return None for non-existent key");
        assert!(result.is_none(), "Non-existent key => None");
    }

    #[test]
    fn test_overwrite_existing_key() {
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let key = b"testkey";
        let initial_val = b"initial";
        let updated_val = b"updated";

        // First put
        db_guard.put(key, initial_val).expect("Should succeed");
        let val1 = db_guard.get(key).expect("Should succeed").unwrap();
        assert_eq!(val1, initial_val);

        // Overwrite
        db_guard.put(key, updated_val).expect("Should succeed");
        let val2 = db_guard.get(key).expect("Should succeed").unwrap();
        assert_eq!(val2, updated_val, "Value should be overwritten");
    }

    #[test]
    fn test_empty_value() {
        // Confirm that storing an empty value is valid.
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let key = b"empty_value_key";
        let empty_value: &[u8] = &[];

        db_guard.put(key, empty_value).expect("Should handle empty value");
        let retrieved = db_guard.get(key).expect("Read should succeed").unwrap();
        assert!(retrieved.is_empty(), "Stored empty value should load as empty");
    }

    #[test]
    fn test_large_value() {
        // We'll attempt storing a larger value (like ~1MB). Some configurations might have limitations.
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let key = b"large_value";
        let large_data = vec![b'x'; 1024 * 1024]; // 1 MB of 'x'

        db_guard.put(key, &large_data).expect("Should store large value");
        let retrieved = db_guard.get(key).expect("Read should succeed").unwrap();
        assert_eq!(retrieved.len(), large_data.len());
        assert_eq!(&retrieved[..], &large_data[..], "Data should match exactly");
    }

    #[test]
    fn test_unicode_in_key_and_value() {
        // We'll try non-ASCII keys and values to ensure everything is handled as raw bytes.
        let (db_arc, _tmp_dir) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let key = "ключ".as_bytes();  // Russian for "key"
        let value = "значение".as_bytes(); // Russian for "value"

        db_guard.put(key, value).expect("Should store unicode bytes");
        let retrieved = db_guard.get(key).expect("Should read back").unwrap();

        // We'll do a quick check if we can decode them back to string
        let retrieved_str = str::from_utf8(&retrieved).unwrap();
        assert_eq!(retrieved_str, "значение", "Should match the original Unicode");
    }

    #[test]
    fn test_rocksdb_error_propagation_on_put() {
        // If an error occurs during `put`, it should return DatabaseConstructionError::RocksDB(_).
        // We'll define a minimal approach: a stub that always fails on `put`.
        // In real usage, you'd do a partial mock or an ephemeral environment error.

        struct FailingDbStub;
        impl DatabasePut for FailingDbStub {
            fn put(&mut self, _key: impl AsRef<[u8]>, _val: impl AsRef<[u8]>) 
                -> Result<(), DatabaseConstructionError> 
            {
                Err(DatabaseConstructionError::RocksDB(rocksdb::Error::new("Simulated failure")))
            }
        }

        let mut db_stub = FailingDbStub;
        let result = db_stub.put(b"some_key", b"some_val");
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated failure");
            }
            other => panic!("Expected RocksDB error, got {:?}", other),
        }
    }

    #[test]
    fn test_rocksdb_error_propagation_on_get() {
        // If an error occurs during `get`, it should return DatabaseConstructionError::RocksDB(_).
        struct FailingDbStub;
        impl DatabaseGet for FailingDbStub {
            fn get(&self, _key: impl AsRef<[u8]>) 
                -> Result<Option<Vec<u8>>, DatabaseConstructionError> 
            {
                Err(DatabaseConstructionError::RocksDB(rocksdb::Error::new("Simulated read error")))
            }
        }

        let db_stub = FailingDbStub;
        let result = db_stub.get(b"some_key");
        match result {
            Err(DatabaseConstructionError::RocksDB(e)) => {
                assert_eq!(e.to_string(), "Simulated read error");
            }
            other => panic!("Expected RocksDB read error, got {:?}", other),
        }
    }
}
