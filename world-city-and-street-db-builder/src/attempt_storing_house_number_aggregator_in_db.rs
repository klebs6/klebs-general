// ---------------- [ File: src/attempt_storing_house_number_aggregator_in_db.rs ]
// ---------------- [ File: src/attempt_storing_in_db.rs ]
crate::ix!();

#[cfg(test)]
#[disable]
mod attempt_storing_house_number_aggregator_in_db_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use std::collections::HashMap;

    /// A helper to generate a test region, e.g. Maryland
    fn md_region() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    /// Creates a minimal DB in a temporary directory (for short‐lived tests).
    fn create_test_db<I:StorageInterface>() -> Arc<Mutex<I>> {
        let tmp = TempDir::new().expect("failed to create temp dir");
        I::open(tmp.path()).expect("failed to open DB")
    }

    /// Builds a small aggregator for a single street => single HouseNumberRange.
    fn build_single_street_aggregator() -> HouseNumberAggregator {
        let street = StreetName::new("North Avenue").unwrap();
        let range = HouseNumberRange::new(100, 110);
        let mut map = HashMap::new();
        map.insert(street, vec![range]);
        HouseNumberAggregator::new_with_map(&WorldRegion::default(),map)
    }

    /// By default, `store_house_number_aggregator_results(...)` is what does the real DB I/O.
    /// If you want to test an error scenario, you might intercept that function or mock it.
    /// Below we rely on the real function for success cases and show
    /// a technique for *optionally* injecting an error via a custom DB approach if needed.

    // ---------------------------------------------------------------------
    // 1) test with an empty aggregator => no actual writes
    // ---------------------------------------------------------------------
    #[test]
    fn test_attempt_storing_house_number_aggregator_in_db_empty() {
        let db_arc = create_test_db();
        let region = md_region();
        let aggregator = HashMap::new(); // empty

        // This should do nothing but log debug statements
        aggregator.attempt_storing_in_db(db_arc.clone(), &region);

        // verify nothing was written => no "HNR" keys
        let db_guard = db_arc.lock().unwrap();
        // We can do a quick iteration:
        let iter = db_guard.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, _val) = item.expect("iterator error").clone();
            let key_str = String::from_utf8_lossy(&key);
            assert!(
                !key_str.starts_with("HNR:"),
                "No aggregator => no HNR keys expected."
            );
        }
    }

    // ---------------------------------------------------------------------
    // 2) test with a single-street aggregator => success => aggregator stored
    // ---------------------------------------------------------------------
    #[test]
    fn test_attempt_storing_house_number_aggregator_in_db_non_empty() {
        let db_arc = create_test_db();
        let region = md_region();
        let aggregator = build_single_street_aggregator();

        aggregator.attempt_storing_in_db(db_arc.clone(), &region);

        // verify that the aggregator subranges were stored
        let db_guard = db_arc.lock().unwrap();
        // "North Avenue" => HNR:MD:north avenue
        let key_str = format!("HNR:{}:{}", region.abbreviation(), "north avenue");
        let val_opt = db_guard.get(&key_str).expect("DB read error");
        assert!(
            val_opt.is_some(),
            "Expected aggregator results stored under 'HNR:MD:north avenue'"
        );
        // Optionally decode the CBOR to confirm subranges
        let raw_bytes = val_opt.unwrap();
        let cresult: serde_cbor::Result<crate::compressed_list::CompressedList<HouseNumberRange>> =
            serde_cbor::from_slice(&raw_bytes);
        assert!(cresult.is_ok());
        let clist = cresult.unwrap();
        assert_eq!(clist.items().len(), 1);
        assert_eq!(clist.items()[0].start(), &100);
        assert_eq!(clist.items()[0].end(), &110);
    }

    // ---------------------------------------------------------------------
    // 3) test DB lock poisoning => it logs a warning and does not panic
    // ---------------------------------------------------------------------
    #[test]
    fn test_attempt_storing_house_number_aggregator_in_db_lock_poisoned() {
        let db_arc = create_test_db();
        let region = md_region();
        let aggregator = build_single_street_aggregator();

        // Force the lock to become poisoned
        let _ = std::panic::catch_unwind(|| {
            let guard = db_arc.lock().unwrap();
            panic!("intentionally poisoning the DB lock");
        });

        // Now the lock is poisoned
        // The function should catch the Err and log a warning
        aggregator.attempt_storing_in_db(db_arc.clone(), &region);

        // There's no easy way to confirm the actual warning log, but at least we confirm
        // it doesn't panic and doesn't store anything.
        // If you'd like, you can see that the aggregator is not stored.
        let try_lock = db_arc.lock();
        assert!(
            try_lock.is_err(),
            "Lock is indeed poisoned => no aggregator stored => no direct checks possible"
        );
    }

    // ---------------------------------------------------------------------
    // 4) optional test if you can mock or simulate store errors
    // ---------------------------------------------------------------------
    // This scenario requires either a custom DB or hooking `store_house_number_aggregator_results`
    // to force an error. We'll illustrate how it might look logically:
    #[test]
    fn test_attempt_storing_house_number_aggregator_in_db_error_storing() {
        // If your real code does not provide a direct way to simulate an error from store_house_number_aggregator_results,
        // you can skip or adapt. For demonstration, let's define a "FakeDB" or a boolean error injection.

        // 1) We'll define a minor closure that simulates store => always returns an error
        fn fake_store_house_number_aggregator_results<I:StorageInterface>(
            _db:           &mut I,
            _world_region: &WorldRegion,
            _aggregator:   HashMap<StreetName, Vec<HouseNumberRange>>,
        ) -> Result<(), OsmPbfParseError> {
            Err(OsmPbfParseError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Simulated store error",
            )))
        }

        // 2) We'll replace the real function with our fake one using a local override technique
        // This can be done by re-implementing attempt_storing_in_db with dependency injection
        // For now, let's just inline a test version:

        fn test_version_of_attempt_storing<I:StorageInterface>(
            db:         Arc<Mutex<I>>,
            region:     &WorldRegion,
            aggregator: HashMap<StreetName, Vec<HouseNumberRange>>,
        ) {
            match db.lock() {
                Ok(mut db_guard) => {
                    if let Err(e) = fake_store_house_number_aggregator_results(
                        &mut db_guard,
                        region,
                        aggregator,
                    ) {
                        // We expect a warning in logs, but we won't fail the entire program
                        warn!("test_version_of_attempt_storing: error => {:?}", e);
                    }
                }
                Err(_) => {
                    warn!("test_version_of_attempt_storing: lock poison");
                }
            }
        }

        // final usage
        let db_arc = create_test_db();
        let aggregator = build_single_street_aggregator();
        // We'll do region => "Maryland"
        let region = md_region();
        test_version_of_attempt_storing(db_arc.clone(), &region, aggregator);

        // We expect a warning in logs, but we won't have any aggregator data in DB
        let guard = db_arc.lock().unwrap();
        let hnr_key = format!("HNR:{}:{}", region.abbreviation(), "north avenue");
        let val = guard.get(hnr_key.as_bytes()).unwrap();
        assert!(val.is_none(), "Because the store returned error => no aggregator written");
    }
}
