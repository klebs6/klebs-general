// ---------------- [ File: src/open_database_at_path.rs ]
// ---------------- [ File: src/open_database_at_path.rs ]
crate::ix!();

pub trait OpenDatabaseAtPath {
    fn open(path: impl AsRef<std::path::Path>) 
        -> Result<Arc<Mutex<Self>>, WorldCityAndStreetDbBuilderError>;
}

impl OpenDatabaseAtPath for Database {

    fn open(path: impl AsRef<std::path::Path>)
        -> Result<Arc<Mutex<Self>>, WorldCityAndStreetDbBuilderError>
    {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(DBCompressionType::Zstd);

        // 1) Use the “colon prefix” transform so that RocksDB
        //    stores an extracted prefix up to the second colon.
        let st = create_colon_prefix_transform();
        opts.set_prefix_extractor(st);

        // Optionally enable prefix bloom filters
        opts.set_memtable_prefix_bloom_ratio(0.1);

        let db = DB::open(&opts, path).map_err(|e| DataAccessError::RocksDB(e))?;

        let db = DatabaseBuilder::default()
            .db(Arc::new(db))
            .build()
            .unwrap();

        Ok(Arc::new(Mutex::new(db)))
    }
}

#[cfg(test)]
mod test_open_database_at_path {
    use super::*;
    use tempfile::TempDir;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// Creates a brand new RocksDB database in a temp directory and verifies success.
    #[traced_test]
    fn test_open_db_in_temp_dir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("testdb");

        // Attempt to open (create) the DB
        let db_result = Database::open(&db_path);
        assert!(
            db_result.is_ok(),
            "Opening DB in a new temp directory should succeed"
        );

        let db_arc = db_result.unwrap();
        let db_guard = db_arc.lock().unwrap();
        // Minimal verification we have a Database object
        assert!(
            db_guard.db().path().exists(),
            "RocksDB path should exist on disk now"
        );
    }

    /// Ensures the DB is created if the directory doesn't exist, 
    /// verifying the `create_if_missing` logic is functioning.
    #[traced_test]
    fn test_create_if_missing_subdir() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let subdir = temp_dir.path().join("nested").join("db_location");

        let db_result = Database::open(&subdir);
        assert!(
            db_result.is_ok(),
            "Database creation should recursively create subdirectories"
        );
        let db_arc = db_result.unwrap();
        let db_guard = db_arc.lock().unwrap();
        assert!(
            db_guard.db().path().exists(),
            "The new DB path should exist after creation"
        );
    }

    /// If we attempt to open a DB path that is a file (not a directory),
    /// RocksDB should fail, as it expects a directory for its data.
    #[traced_test]
    fn test_open_db_with_file_path_fails() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("not_a_directory");
        {
            // Create an empty file at `file_path`.
            File::create(&file_path).expect("Failed to create file in temp dir");
        }

        match Database::open(&file_path) {
            Ok(db) => {
                panic!("Expected a RocksDB error when using a file path");
            }
            Err(e) => {
                // The exact error message might differ across environments,
                // but we'll check for at least a partial phrase if needed.
                assert!(
                    e.to_string().contains("Invalid argument")
                    || e.to_string().contains("is not a directory")
                    || e.to_string().contains("Failed to create RocksDB directory")
                    , "Expected an error indicating that the path is invalid: {}", e
                );
            }
        }
    }

    /// Attempts to open a DB in a read-only directory should fail (on most systems).
    #[traced_test]
    fn test_open_db_in_read_only_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let read_only_dir = temp_dir.path().join("readonly");
        fs::create_dir_all(&read_only_dir).expect("Failed to create read-only dir");
        
        // Make it read-only by removing write permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&read_only_dir).unwrap().permissions();
            perms.set_mode(0o555); // read & execute only
            fs::set_permissions(&read_only_dir, perms).unwrap();
        }

        // Attempt to open (create) DB here
        let db_result = Database::open(&read_only_dir);
        #[cfg(unix)]
        assert!(
            db_result.is_err(),
            "Opening DB in a read-only directory should fail under Unix"
        );
        #[cfg(not(unix))]
        {
            // On non-Unix systems, read-only semantics might differ
            // We won't enforce the error strictly, but we'll log it.
            if db_result.is_err() {
                eprintln!("Non-Unix: opening DB in read-only dir also failed, as expected");
            }
        }
    }

    /// Verifies that the prefix transform and bloom filter options are set without error.
    /// We can't easily introspect RocksDB's internal state to confirm, but we can open
    /// the DB to ensure no panic or config error arises from these advanced options.
    #[traced_test]
    fn test_open_db_with_prefix_transform_and_bloom_filter_enabled() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("prefix_test");

        // This just ensures open() doesn't fail from the custom prefix transform or bloom ratio.
        let db_result = Database::open(&db_path);
        assert!(
            db_result.is_ok(),
            "DB should open successfully with prefix transform and bloom filter enabled"
        );
        let db_arc = db_result.unwrap();
        let mut db_guard = db_arc.lock().unwrap();

        // We can do a minimal check: put & get a prefix-based key
        // This won't fully confirm the prefix transform is active, but ensures no crash
        // usage with that feature.
        let key = b"ABC:123";
        let val = b"test_value";
        db_guard.put(key, val).unwrap();
        let retrieved = db_guard.get(key).unwrap().unwrap();
        assert_eq!(
            retrieved, val,
            "Should be able to store & retrieve with the prefix-based transform"
        );
    }
}
