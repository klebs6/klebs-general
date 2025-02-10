// ---------------- [ File: src/prefix_transform.rs ]
// ---------------- [ File: src/prefix_transform.rs ]
crate::ix!();

/// Create a `SliceTransform` that extracts everything **up to and including** the second `':'`
/// in each key. If the key has fewer than two colons, we return the entire key.
pub fn create_colon_prefix_transform() -> SliceTransform {
    SliceTransform::create(
        "my_colon_prefix_transform",
        |key: &[u8]| {
            let mut colon_count = 0;
            for (i, &byte) in key.iter().enumerate() {
                if byte == b':' {
                    colon_count += 1;
                    if colon_count == 2 {
                        // return slice up to and including that second colon
                        return &key[..=i];
                    }
                }
            }
            // If fewer than 2 colons => the entire key is the “prefix”
            key
        },
        // The domain check is optional; at minimum we confirm it’s non-empty.
        Some(|k: &[u8]| !k.is_empty()),
    )
}

#[cfg(test)]
mod prefix_transform_tests {
    use super::*;

    #[traced_test]
    fn test_dynamic_colon_prefix_extractor() {
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path()).expect("open DB");
        let mut db_guard = db.lock().unwrap();

        db_guard.put(b"C2Z:US:baltimore", b"data").unwrap();
        db_guard.put(b"C2Z:US:some:extra", b"data").unwrap();
        db_guard.put(b"C2Z:EU:london", b"data").unwrap();
        db_guard.put(b"C2Z:US", b"partial").unwrap();
        db_guard.put(b"Z2C:US:21201", b"data").unwrap();

        // 1) prefix="C2Z:US:"
        // Must call db_guard.db().prefix_iterator(...) 
        let mut iter = db_guard.prefix_iterator(b"C2Z:US:");
        let mut found = Vec::new();
        while let Some(Ok((k, v))) = iter.next() {
            // We only store the key string, so we can do .contains() easily.
            let key_str = String::from_utf8_lossy(&k).to_string();
            found.push(key_str);
        }
        // We expect "C2Z:US:baltimore" and "C2Z:US:some:extra"
        assert_eq!(found.len(), 2);
        assert!(found.contains(&"C2Z:US:baltimore".to_string()));
        assert!(found.contains(&"C2Z:US:some:extra".to_string()));

        // 2) prefix="Z2C:US:"
        let mut iter2 = db_guard.prefix_iterator(b"Z2C:US:");
        let mut found2 = Vec::new();
        while let Some(Ok((k, _v))) = iter2.next() {
            let key_str = String::from_utf8_lossy(&k).to_string();
            found2.push(key_str);
        }
        // Should see only "Z2C:US:21201"
        assert_eq!(found2, vec!["Z2C:US:21201"]);
    }

    #[traced_test]
    fn test_my_colon_prefix_transform() {
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path()).expect("open DB with prefix transform");
        let mut db_guard = db.lock().unwrap();

        // Insert keys
        db_guard.put(b"C2Z:US:baltimore", b"baltimore_data").unwrap();
        db_guard.put(b"C2Z:US:annapolis", b"annapolis_data").unwrap();
        db_guard.put(b"Z2C:US:21201",     b"zip_data").unwrap();
        db_guard.put(b"C2Z:EU:london",   b"london_data").unwrap();
        db_guard.put(b"C2Z:US",          b"partial_data").unwrap();
        db_guard.put(b"C2Z:US:some:extra:stuff", b"extra_data").unwrap();

        // prefix="C2Z:US:"
        let mut iter = db_guard.prefix_iterator(b"C2Z:US:");
        let mut found = Vec::new();
        while let Some(Ok((k, _v))) = iter.next() {
            found.push(String::from_utf8_lossy(&k).to_string());
        }

        // We expect 3 matches: "C2Z:US:baltimore", "C2Z:US:annapolis", "C2Z:US:some:extra:stuff"
        assert_eq!(found.len(), 3);
        assert!(found.contains(&"C2Z:US:baltimore".to_string()));
        assert!(found.contains(&"C2Z:US:annapolis".to_string()));
        assert!(found.contains(&"C2Z:US:some:extra:stuff".to_string()));

        // Check prefix="C2Z:US" => we only find "C2Z:US" (the key that has <2 colons)
        found.clear();
        let mut iter2 = db_guard.prefix_iterator(b"C2Z:US");
        while let Some(Ok((k, _v))) = iter2.next() {
            found.push(String::from_utf8_lossy(&k).to_string());
        }

        assert_eq!(found.len(), 1);
        assert_eq!(found[0], "C2Z:US");
    }
}
