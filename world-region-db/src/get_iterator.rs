crate::ix!();

pub trait GetIterator {

    fn iterator<'a: 'b, 'b>(
        &'a self,
        mode: rocksdb::IteratorMode,
    ) -> rocksdb::DBIteratorWithThreadMode<'b, rocksdb::DB>;
}

impl GetIterator for Database {

    fn iterator<'a: 'b, 'b>(
        &'a self,
        mode: rocksdb::IteratorMode,
    ) -> rocksdb::DBIteratorWithThreadMode<'b, rocksdb::DB> {
        self.db().iterator(mode)
    }
}

#[cfg(test)]
mod get_iterator_traits_tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::{Arc, Mutex};

    fn create_db<I: StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp = TempDir::new().expect("tempdir");
        let db = I::open(temp.path()).expect("db open");
        (db, temp)
    }

    /// Helper to write a few key-value pairs into the DB
    fn insert_test_data(db_arc: &Arc<Mutex<Database>>) {
        let mut db_guard = db_arc.lock().expect("lock DB");
        db_guard.put("alpha", b"value_alpha").unwrap();
        db_guard.put("beta", b"value_beta").unwrap();
        db_guard.put("prefix:one", b"prefix_one").unwrap();
        db_guard.put("prefix:two", b"prefix_two").unwrap();
        db_guard.put("prefix:zzz", b"prefix_zzz").unwrap();
    }

    #[traced_test]
    fn test_iterator_start() {
        let (db_arc, _tempdir) = create_db();
        insert_test_data(&db_arc);

        let db_guard = db_arc.lock().expect("lock DB");
        // We'll do a forward iteration from the start using DBIteratorWithThreadMode
        let mut iter = db_guard.iterator(rocksdb::IteratorMode::Start);
        let mut found = vec![];
        while let Some(Ok((k, v))) = iter.next() {
            found.push((
                String::from_utf8_lossy(&k).to_string(),
                String::from_utf8_lossy(&v).to_string(),
            ));
        }
        assert_eq!(found.len(), 5, "we inserted 5 key-value pairs");
    }

    /// Demonstrates *reverse iteration* using the raw iterator.
    /// DBIteratorWithThreadMode does not provide `.prev()`, so we use `db.raw_iterator()`
    /// and call `seek_to_last()` then `.prev()`.
    #[traced_test]
    fn test_raw_iterator_reverse() {
        let (db_arc, _tempdir) = create_db();
        insert_test_data(&db_arc);

        let db_guard = db_arc.lock().unwrap();
        // Acquire a raw iterator:
        let mut raw_iter = db_guard.db().raw_iterator();

        // Position it at the *last* key
        raw_iter.seek_to_last();
        assert!(
            raw_iter.valid(),
            "After seek_to_last(), iterator should be valid if DB is not empty"
        );

        let mut reversed_pairs = Vec::new();

        // While valid, gather key/value, then `.prev()`
        while raw_iter.valid() {
            // raw_iter.key() and raw_iter.value() return Option<&[u8]>
            let k = raw_iter.key().expect("some key bytes");
            let v = raw_iter.value().expect("some value bytes");
            let k_str = String::from_utf8_lossy(k).to_string();
            let v_str = String::from_utf8_lossy(v).to_string();

            reversed_pairs.push((k_str, v_str));

            raw_iter.prev(); // move to the previous entry
        }

        // Now reversed_pairs should contain all 5 pairs in descending key order
        assert_eq!(
            reversed_pairs.len(),
            5,
            "Expected to see all 5 key-value pairs in reverse order"
        );

        // For demonstration, let's see the last key we encountered
        // might be "alpha" if 'alpha' is the lexicographically smallest:
        let last_in_reverse = reversed_pairs.last().unwrap().0.clone();
        println!("The lexicographically first key was encountered last: {}", last_in_reverse);
    }
}
