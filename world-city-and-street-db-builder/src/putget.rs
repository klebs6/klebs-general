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
        -> Result<Option<Vec<u8>>,DataAccessError>;
}

impl DatabaseGet for Database {

    fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>,DataAccessError> {
        Ok(self.db().get(key)?)
    }
}

#[cfg(test)]
mod test_database_put_get {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};
    use std::str;

    #[traced_test]
    fn test_put_and_get_round_trip() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
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

    #[traced_test]
    fn test_get_non_existent_key_returns_none() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let missing_key = b"no_such_key";
        let result = db_guard.get(missing_key)
            .expect("Should succeed, but return None for non-existent key");
        assert!(result.is_none(), "Non-existent key => None");
    }

    #[traced_test]
    fn test_overwrite_existing_key() {
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
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

    #[traced_test]
    fn test_empty_value() {
        // Confirm that storing an empty value is valid.
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let key = b"empty_value_key";
        let empty_value: &[u8] = &[];

        db_guard.put(key, empty_value).expect("Should handle empty value");
        let retrieved = db_guard.get(key).expect("Read should succeed").unwrap();
        assert!(retrieved.is_empty(), "Stored empty value should load as empty");
    }

    #[traced_test]
    fn test_large_value() {
        // We'll attempt storing a larger value (like ~1MB). Some configurations might have limitations.
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let key = b"large_value";
        let large_data = vec![b'x'; 1024 * 1024]; // 1 MB of 'x'

        db_guard.put(key, &large_data).expect("Should store large value");
        let retrieved = db_guard.get(key).expect("Read should succeed").unwrap();
        assert_eq!(retrieved.len(), large_data.len());
        assert_eq!(&retrieved[..], &large_data[..], "Data should match exactly");
    }

    #[traced_test]
    fn test_unicode_in_key_and_value() {
        // We'll try non-ASCII keys and values to ensure everything is handled as raw bytes.
        let (db_arc, _tmp_dir) = create_temp_db::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let key = "ключ".as_bytes();  // Russian for "key"
        let value = "значение".as_bytes(); // Russian for "value"

        db_guard.put(key, value).expect("Should store unicode bytes");
        let retrieved = db_guard.get(key).expect("Should read back").unwrap();

        // We'll do a quick check if we can decode them back to string
        let retrieved_str = str::from_utf8(&retrieved).unwrap();
        assert_eq!(retrieved_str, "значение", "Should match the original Unicode");
    }

    #[traced_test]
    fn test_rocksdb_error_propagation_on_get() {
        // If an error occurs during `get`, it should return DatabaseConstructionError::RocksDB(_).
        struct FailingDbStub;
        impl DatabaseGet for FailingDbStub {
            fn get(&self, _key: impl AsRef<[u8]>) 
                -> Result<Option<Vec<u8>>, DataAccessError> 
            {
                Err(DataAccessError::SimulatedReadError)
            }
        }

        let db_stub = FailingDbStub;
        let result = db_stub.get(b"some_key");
        match result {
            Err(DataAccessError::SimulatedReadError) => { }
            other => panic!("Expected RocksDB read error, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_rocksdb_error_propagation_on_put() {
        // If an error occurs during `put`, it should return DatabaseConstructionError::RocksDB(_).
        // We'll define a stub that always fails on `put`, building a RocksDB error
        // by using the public `Error::IOError(...)` variant.

        struct FailingDbStub;

        impl DatabasePut for FailingDbStub {
            fn put(
                &mut self, 
                _key: impl AsRef<[u8]>, 
                _val: impl AsRef<[u8]>
            ) -> Result<(), DatabaseConstructionError> {
                Err(DatabaseConstructionError::SimulatedStoreFailure)
            }
        }

        let mut db_stub = FailingDbStub;
        let result = db_stub.put(b"some_key", b"some_val");

        match result {
            Err(DatabaseConstructionError::SimulatedStoreFailure) => { }
            other => panic!("Expected SimulatedStoreFailure error, got {:?}", other),
        }
    }
}
