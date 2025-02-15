// ---------------- [ File: src/get_prefix_iterator.rs ]
crate::ix!();

pub trait GetPrefixIterator {
    fn prefix_iterator<'a: 'b, 'b, P: AsRef<[u8]>>(
        &'a self,
        prefix: P
    ) -> DBIteratorWithThreadMode<'b, DB>;
}

/// Because older versions of `rust-rocksdb` do not have `prefix_iterator_opt`,
/// we do `iterator_opt(From(...), ...)` with a custom `ReadOptions` that sets
/// `prefix_same_as_start(true)`.
///
/// This positions the iterator at `>= prefix` and, with the prefix extractor,
/// it should terminate once the prefix diverges.
impl GetPrefixIterator for Database {
    fn prefix_iterator<'a: 'b, 'b, P: AsRef<[u8]>>(
        &'a self,
        prefix: P
    ) -> DBIteratorWithThreadMode<'b, DB> {
        let prefix_bytes = prefix.as_ref();

        // We'll set up read options in total‐order, plus prefix-same-as-start.
        let mut read_opts = rocksdb::ReadOptions::default();
        // This is the crucial setting:
        read_opts.set_prefix_same_as_start(true);

        // We start at the first key >= `prefix_bytes`, going forward.
        let mode = rocksdb::IteratorMode::From(prefix_bytes, rocksdb::Direction::Forward);
        self.db().iterator_opt(mode, read_opts)
    }
}

#[cfg(test)]
mod prefix_transform_tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_dynamic_colon_prefix_extractor() {
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path()).expect("open DB");
        let mut db_guard = db.lock().unwrap();

        // Insert sample keys
        db_guard.put(b"C2Z:US:baltimore", b"data").unwrap();
        db_guard.put(b"C2Z:US:some:extra", b"data").unwrap();
        db_guard.put(b"C2Z:EU:london", b"data").unwrap();
        db_guard.put(b"C2Z:US", b"partial").unwrap();
        db_guard.put(b"Z2C:US:21201", b"data").unwrap();

        // 1) prefix="C2Z:US:"
        {
            let mut iter = db_guard.prefix_iterator("C2Z:US:");
            let mut found = Vec::new();
            while let Some(Ok((k, _v))) = iter.next() {
                found.push(String::from_utf8_lossy(&k).to_string());
            }
            // Expect "C2Z:US:baltimore" and "C2Z:US:some:extra"
            assert_eq!(found.len(), 2);
            assert!(found.contains(&"C2Z:US:baltimore".to_string()));
            assert!(found.contains(&"C2Z:US:some:extra".to_string()));
        }

        // 2) prefix="Z2C:US:"
        {
            let mut iter2 = db_guard.prefix_iterator("Z2C:US:");
            let mut found2 = Vec::new();
            while let Some(Ok((k, _v))) = iter2.next() {
                found2.push(String::from_utf8_lossy(&k).to_string());
            }
            // Should see only "Z2C:US:21201"
            assert_eq!(found2, vec!["Z2C:US:21201"]);
        }
    }

    #[test]
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
        {
            let mut iter = db_guard.prefix_iterator("C2Z:US:");
            let mut found = Vec::new();
            while let Some(Ok((k, _v))) = iter.next() {
                found.push(String::from_utf8_lossy(&k).to_string());
            }

            // Should match anything whose slice transform is "C2Z:US:"
            // i.e. keys that contain *at least* 2 colons, and the second colon is at the same position
            // as "C2Z:US:". We expect:
            //   * "C2Z:US:baltimore" => transform is "C2Z:US:"
            //   * "C2Z:US:annapolis" => transform is "C2Z:US:"
            //   * "C2Z:US:some:extra:stuff" => transform is "C2Z:US:"
            // => total 3 results
            assert_eq!(found.len(), 3);
            assert!(found.contains(&"C2Z:US:baltimore".to_string()));
            assert!(found.contains(&"C2Z:US:annapolis".to_string()));
            assert!(found.contains(&"C2Z:US:some:extra:stuff".to_string()));
        }

        // prefix="C2Z:US"
        {
            let mut iter2 = db_guard.prefix_iterator("C2Z:US");
            let mut found = Vec::new();
            while let Some(Ok((k, _v))) = iter2.next() {
                found.push(String::from_utf8_lossy(&k).to_string());
            }

            // Because there's exactly 1 colon, the transform returns the entire key 
            // if it has <2 colons. The key "C2Z:US" has exactly 1 colon => transform is "C2Z:US".
            // We only want the key "C2Z:US" itself. 
            // Meanwhile, "C2Z:US:baltimore" transforms to "C2Z:US:", which doesn't match "C2Z:US".
            // So we expect exactly 1 match: "C2Z:US"
            assert_eq!(found.len(), 1);
            assert_eq!(found[0], "C2Z:US");
        }
    }
}
