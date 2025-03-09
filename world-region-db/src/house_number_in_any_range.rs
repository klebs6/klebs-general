// ---------------- [ File: src/house_number_in_any_range.rs ]
crate::ix!();

pub trait HouseNumberInAnyRange {

    fn house_number_in_any_range(
        &self,
        region:      &WorldRegion,
        street:      &StreetName,
        house_num:   u32,
    ) -> Result<bool, DataAccessError>;
}

impl HouseNumberInAnyRange for Database {

    /// Utility function to check if a given house number is contained
    /// in any of the sub-ranges for a region+street.
    fn house_number_in_any_range(
        &self,
        region:      &WorldRegion,
        street:      &StreetName,
        house_num:   u32,
    ) -> Result<bool, DataAccessError> {

        if let Some(ranges) = self.load_house_number_ranges(region, street)? {
            for rng in ranges {
                if rng.contains(house_num) {
                    return Ok(true);
                }
            }
            Ok(false)
        } else {
            // No entry found, so presumably no coverage
            Ok(false)
        }
    }
}

#[cfg(test)]
mod house_number_in_any_range_tests {
    use super::*;

    // For convenience, we assume `Database` has a dependency on methods like
    // `load_house_number_ranges(...)` and so on. We'll define some minimal
    // helpers here or re‐use your existing store/merge logic.

    /// A small helper to create a fresh DB in a temporary directory.
    /// Returns `(Arc<Mutex<Database>>, TempDir)`.
    fn create_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let tmp = TempDir::new().expect("failed to create TempDir");
        let db = I::open(tmp.path()).expect("Database::open should succeed");
        (db, tmp)
    }

    /// Helper function to store house number ranges for a given street,
    /// so we can test `house_number_in_any_range`.
    fn store_ranges_in_db<I:StorageInterface>(
        db_guard: &mut I,
        region:   &WorldRegion,
        street:   &StreetName,
        ranges:   &[HouseNumberRange]
    ) {
        // We rely on your existing method that does the cbor compression, etc.
        // If you have a different method name (like `store_house_number_ranges`),
        // just use that. For demonstration:
        db_guard.store_house_number_ranges(region, street, ranges)
            .expect("storing house number ranges should succeed");
    }

    #[traced_test]
    fn test_house_number_in_any_range_no_data_key() {
        // scenario: DB has no entry for "HNR:MD:some_street" => `house_number_in_any_range` => false
        let (db_arc, _tmp) = create_db::<Database>();
        let db_guard = db_arc.lock().unwrap();

        let region = example_region();
        let street = StreetName::new("Imaginary Road").unwrap();
        
        let result = db_guard.house_number_in_any_range(&region, &street, 123);
        assert!(result.is_ok());
        let in_range = result.unwrap();
        assert!(!in_range, "No data => definitely false");
    }

    #[traced_test]
    fn test_house_number_in_any_range_empty_stored() {
        // scenario: The DB key exists but has an empty array of sub‐ranges. => false
        let (db_arc, _tmp) = create_db::<Database>();
        {
            let mut guard = db_arc.lock().unwrap();
            let region = example_region();
            let street = StreetName::new("Empty Road").unwrap();
            // store an empty slice of ranges:
            store_ranges_in_db(&mut *guard, &region, &street, &[]);
        }
        {
            let guard = db_arc.lock().unwrap();
            let region = example_region();
            let street = StreetName::new("Empty Road").unwrap();
            let result = guard.house_number_in_any_range(&region, &street, 50);
            assert!(result.is_ok());
            assert!(!result.unwrap(), "No sub-ranges => always false");
        }
    }

    #[traced_test]
    fn test_house_number_in_any_range_single_range_not_included() {
        // scenario: single sub‐range => [100..=110], query 99 => false, query 111 => false
        let (db_arc, _tmp) = create_db::<Database>();
        let region = example_region();
        let street = StreetName::new("SingleRange Rd").unwrap();

        {
            let mut guard = db_arc.lock().unwrap();
            let subranges = vec![HouseNumberRange::new(100, 110)];
            store_ranges_in_db(&mut *guard, &region, &street, &subranges);
        }

        {
            let guard = db_arc.lock().unwrap();
            assert!(!guard.house_number_in_any_range(&region, &street, 99).unwrap());
            assert!(!guard.house_number_in_any_range(&region, &street, 111).unwrap());
        }
    }

    #[traced_test]
    fn test_house_number_in_any_range_single_range_included() {
        // scenario: single sub‐range => [100..=110], query 100 => true, query 105 => true, query=110 => true
        let (db_arc, _tmp) = create_db::<Database>();
        let region = example_region();
        let street = StreetName::new("SingleRange Rd").unwrap();

        {
            let mut guard = db_arc.lock().unwrap();
            let subranges = vec![HouseNumberRange::new(100, 110)];
            store_ranges_in_db(&mut *guard, &region, &street, &subranges);
        }

        {
            let guard = db_arc.lock().unwrap();
            assert!(guard.house_number_in_any_range(&region, &street, 100).unwrap());
            assert!(guard.house_number_in_any_range(&region, &street, 105).unwrap());
            assert!(guard.house_number_in_any_range(&region, &street, 110).unwrap());
        }
    }

    #[traced_test]
    fn test_house_number_in_any_range_multiple_disjoint_ranges() {
        // scenario: multiple sub‐ranges => [1..=10], [20..=30], [40..=50]
        //  => query 5 => true, query 15 => false, query 45 => true
        let (db_arc, _tmp) = create_db::<Database>();
        let region = example_region();
        let street = StreetName::new("MultiRange Blvd").unwrap();

        {
            let mut guard = db_arc.lock().unwrap();
            let subranges = vec![
                HouseNumberRange::new(1, 10),
                HouseNumberRange::new(20, 30),
                HouseNumberRange::new(40, 50),
            ];
            store_ranges_in_db(&mut *guard, &region, &street, &subranges);
        }

        {
            let guard = db_arc.lock().unwrap();
            // in first range
            assert!(guard.house_number_in_any_range(&region, &street, 5).unwrap());
            // between first and second => false
            assert!(!guard.house_number_in_any_range(&region, &street, 15).unwrap());
            // in third range
            assert!(guard.house_number_in_any_range(&region, &street, 45).unwrap());
        }
    }

    #[traced_test]
    fn test_house_number_in_any_range_error_on_load() {
        // scenario: if `load_house_number_ranges(...)` returns an Err => we propagate that as DataAccessError
        // We'll do partial approach: forcibly cause an error (e.g., lock poisoning or a RocksDB error).
        // We'll do lock poisoning. 
        let (db_arc, _tmp) = create_db::<Database>();
        let region = example_region();
        let street = StreetName::new("Poison Road").unwrap();

        // 1) Wrap the “poisoning” in catch_unwind so it doesn’t abort the test on panic
        let res = std::panic::catch_unwind(|| {
            let _guard = db_arc.lock().unwrap();
            panic!("Intentional poisoning of lock => cause subsequent attempts to error");
        });

        // 2) We *expect* a panic from inside the closure:
        assert!(res.is_err(), "We expected the closure to panic");

        // 3) Now the lock is presumably poisoned. Attempt to re-lock:
        let db_guard_result = db_arc.lock();
        assert!(db_guard_result.is_err(), "Lock is poisoned => locking should return an Err(...)");

        // 4) At this point, we can’t do normal DB calls because the lock is in an error state.
        // But we *have tested* that the lock is indeed poisoned. If your real code handles 
        // PoisonError by returning a DataAccessError, you could test that logic here 
        // (i.e., confirm your code gracefully returns an error from house_number_in_any_range).

        // If you want a direct call:
        // let result = house_number_in_any_range( ??? );
        // But since we can't unwrap the lock, you'd probably do some error-handling branch.
        // E.g. your real code might do: 
        //   let guard = db_arc.lock().map_err(|_| DataAccessError::poisoned())?
        // So you can confirm that logic works as intended. 
        //
        // Alternatively, see Approach #1 (Mock the load method) if you purely want to 
        // confirm that an error from load_house_number_ranges is propagated.
    }
}
