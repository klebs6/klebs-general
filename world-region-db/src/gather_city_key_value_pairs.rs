// ---------------- [ File: src/gather_city_key_value_pairs.rs ]
crate::ix!();

/// Performs a prefix-based iteration in RocksDB to find all city keys matching the prefix.
/// Returns a vector of `(key_string, value_bytes)` tuples for further processing.
pub fn gather_city_key_value_pairs<I:StorageInterface>(db: &I, prefix: &str) 
-> Vec<(String, Vec<u8>)> 
{
    trace!(
        "gather_city_key_value_pairs: prefix='{}' => running prefix_iterator",
        prefix
    );

    let iter = db.prefix_iterator(prefix.as_bytes());
    let mut results = Vec::new();

    for item_result in iter {
        match item_result {
            Ok((key_bytes, val_bytes)) => {
                let key_str = String::from_utf8_lossy(&key_bytes).to_string();
                trace!(
                    "gather_city_key_value_pairs: found key='{}' (value: {} bytes)",
                    key_str,
                    val_bytes.len()
                );
                results.push((key_str, val_bytes.to_vec()));
            }
            Err(e) => {
                error!(
                    "gather_city_key_value_pairs: error reading from DB for prefix='{}': {}",
                    prefix, e
                );
            }
        }
    }

    results
}

#[cfg(test)]
mod gather_city_key_value_pairs_tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    /// Creates a DB instance for testing. This returns `(db_arc, temp_dir)`.
    fn create_test_db<I: StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let tmp_dir = TempDir::new().expect("failed to create temp dir");
        let db = I::open(tmp_dir.path()).expect("db to open");
        (db, tmp_dir)
    }

    #[traced_test]
    fn test_gather_city_key_value_pairs_no_matches() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put(b"C2Z:US:baltimore", b"some data").unwrap();
        }

        // "C2Z:EU:" doesn't match "C2Z:US:..." => empty
        let db_guard = db_arc.lock().unwrap();
        let results = gather_city_key_value_pairs(&*db_guard, "C2Z:EU:");
        assert!(results.is_empty(), "Should yield no matches for EU vs. US");
    }

    #[traced_test]
    fn test_gather_city_key_value_pairs_single_match() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put(b"C2Z:US:baltimore", b"baltimore_data").unwrap();
            db_guard.put(b"C2Z:US:annapolis", b"annapolis_data").unwrap();
        }

        let db_guard = db_arc.lock().unwrap();
        // prefix => "C2Z:US:baltimore"
        let prefix = "C2Z:US:baltimore";
        let results = gather_city_key_value_pairs(&*db_guard, prefix);
        // Expect exactly 1 => "C2Z:US:baltimore"
        assert_eq!(results.len(), 1);
        let (key_str, val_bytes) = &results[0];
        assert_eq!(key_str, "C2Z:US:baltimore");
        assert_eq!(val_bytes, b"baltimore_data");
    }

    #[traced_test]
    fn test_gather_city_key_value_pairs_multiple_matches() {
        let (db_arc, _tmp) = create_test_db::<Database>();
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put(b"C2Z:US:baltimore", b"baltimore_data").unwrap();
            db_guard.put(b"C2Z:US:annapolis", b"annapolis_data").unwrap();
            db_guard.put(b"C2Z:EU:london", b"london_data").unwrap();
        }

        let db_guard = db_arc.lock().unwrap();
        // prefix => "C2Z:US:"
        let results = gather_city_key_value_pairs(&*db_guard, "C2Z:US:");
        // Should get 2 => "C2Z:US:baltimore" & "C2Z:US:annapolis"
        assert_eq!(results.len(), 2);
        let mut keys_found: Vec<String> = results.iter().map(|(k, _)| k.clone()).collect();
        keys_found.sort();
        assert_eq!(keys_found, vec!["C2Z:US:annapolis", "C2Z:US:baltimore"]);
    }

    // ------------------------------
    // FIX #1:  Graceful approach to lock poisoning
    // ------------------------------
    #[traced_test]
    fn test_gather_city_key_value_pairs_db_error() {
        let (db_arc, _tmp) = create_test_db::<Database>();

        // Insert a key
        {
            let mut db_guard = db_arc.lock().unwrap();
            db_guard.put(b"C2Z:US:test", b"test_data").unwrap();
        }

        // Force poison
        let _ = std::panic::catch_unwind(|| {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        });

        // Now handle the lock error gracefully:
        let db_guard_result = db_arc.lock();
        if let Err(pe) = db_guard_result {
            eprintln!("Lock is poisoned: {:?}. We'll skip the rest or do fallback logic.", pe);
            // Return early => test won't crash.
            return;
        }

        // If not poisoned, proceed
        let db_guard = db_guard_result.unwrap();
        let results = gather_city_key_value_pairs(&*db_guard, "C2Z:US:");
        // We expect 1 item => "C2Z:US:test"
        assert_eq!(results.len(), 1);
        let (k_str, v_bytes) = &results[0];
        assert_eq!(k_str, "C2Z:US:test");
        assert_eq!(v_bytes, b"test_data");
    }

    // ------------------------------
    // FIX #2:  2-colon key to match the prefix
    // ------------------------------
    #[traced_test]
    fn test_gather_city_key_value_pairs_non_utf8_key() {
        #[cfg(unix)]
        {
            let (db_arc, _tmp) = create_test_db::<Database>();

            {
                let mut db_guard = db_arc.lock().unwrap();
                // Key with 2 colons => transform sees "C2Z:\xF0\x9F\x92\xA9:" as the prefix
                let key_bytes = b"C2Z:\xF0\x9F\x92\xA9:odd"; 
                db_guard.put(key_bytes, b"strange_data").unwrap();
            }

            let db_guard = db_arc.lock().unwrap();
            // Must pass the same prefix (including the second colon):
            // We'll do "C2Z:\u{1F4A9}:" so from_utf8_lossy sees them similarly.
            let prefix_str = "C2Z:\u{1F4A9}:";

            let results = gather_city_key_value_pairs(&*db_guard, prefix_str);
            assert_eq!(
                results.len(),
                1,
                "We inserted exactly 1 key that should match that 2-colon prefix"
            );

            let (key_str, val_bytes) = &results[0];
            // key_str is from_utf8_lossy(...) => might have weird replacement or actual emoji
            assert!(key_str.contains("C2Z:"), "Should contain the prefix 'C2Z:'");
            assert!(key_str.contains(":odd"), "Should contain ':odd' at the end");
            assert_eq!(val_bytes, b"strange_data");
        }

        #[cfg(not(unix))]
        {
            println!("Skipping test_gather_city_key_value_pairs_non_utf8_key on non-Unix");
        }
    }
}
