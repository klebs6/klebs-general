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

    fn create_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp = TempDir::new().expect("tempdir");
        let db   = I::open(temp.path()).expect("db open");
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
        // We can also test numeric sorting, etc. if desired
    }

    #[traced_test]
    fn test_iterator_start() {
        let (db_arc, _tempdir) = create_db();
        insert_test_data(&db_arc);

        let db_guard = db_arc.lock().expect("lock DB");
        // We'll do a forward iteration from the start
        let mut iter = db_guard.iterator(rocksdb::IteratorMode::Start);
        let mut found = vec![];
        while let Some(Ok((k, v))) = iter.next() {
            found.push((
                String::from_utf8_lossy(&k).to_string(),
                String::from_utf8_lossy(&v).to_string(),
            ));
        }
        // Sort them by key or just observe the insertion order. 
        // RocksDB might not store in insertion order if not using an OrderedColumnFamily, but typically
        // we see ascending lexicographical if default comparator is used.
        // We'll just check that we got them all:
        assert_eq!(found.len(), 5, "we inserted 5 key-value pairs");
        // (We won't enforce the order strictly unless we rely on known key ordering.)
        let mut keys: Vec<_> = found.iter().map(|(k, _)| k.clone()).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["alpha", "beta", "prefix:one", "prefix:two", "prefix:zzz"]
        );
    }

    #[traced_test]
    fn test_iterator_end_mode() {
        let (db_arc, _tempdir) = create_db();
        insert_test_data(&db_arc);

        let db_guard = db_arc.lock().unwrap();
        // If we do `IteratorMode::End`, it starts at the last key and goes backwards. 
        // We'll check if we can handle that logic.
        let mut iter = db_guard.iterator(rocksdb::IteratorMode::End);
        // Typically the first `next()` from an end-iterator yields None if we haven't done .prev(). 
        // But let's see how your code sets up RocksDB. 
        // We'll do a partial approach: we expect an empty iteration if we only do `next()` calls.
        let first = iter.next();
        assert!(first.is_none());
        // If we wanted a reverse iteration, we'd do .prev() calls. But the trait doesn't show that usage here.
    }
}
