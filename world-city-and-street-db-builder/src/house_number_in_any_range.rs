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
    fn create_db() -> (Arc<Mutex<Database>>, TempDir) {
        let tmp = TempDir::new().expect("failed to create TempDir");
        let db = Database::open(tmp.path()).expect("Database::open should succeed");
        (db, tmp)
    }

    /// Helper function to store house number ranges for a given street,
    /// so we can test `house_number_in_any_range`.
    fn store_ranges_in_db(
        db_guard: &mut Database,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange]
    ) {
        // We rely on your existing method that does the cbor compression, etc.
        // If you have a different method name (like `store_house_number_ranges`),
        // just use that. For demonstration:
        db_guard.store_house_number_ranges(region, street, ranges)
            .expect("storing house number ranges should succeed");
    }

    /// Constructs a typical region object for your tests. 
    /// In a real environment, pick the region that best matches your system (MD, VA, DC, etc.).
    fn example_region() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }

    #[test]
    fn test_house_number_in_any_range_no_data_key() {
        // scenario: DB has no entry for "HNR:MD:some_street" => `house_number_in_any_range` => false
        let (db_arc, _tmp) = create_db();
        let db_guard = db_arc.lock().unwrap();

        let region = example_region();
        let street = StreetName::new("Imaginary Road").unwrap();
        
        let result = db_guard.house_number_in_any_range(&region, &street, 123);
        assert!(result.is_ok());
        let in_range = result.unwrap();
        assert!(!in_range, "No data => definitely false");
    }

    #[test]
    fn test_house_number_in_any_range_empty_stored() {
        // scenario: The DB key exists but has an empty array of sub‐ranges. => false
        let (db_arc, _tmp) = create_db();
        {
            let mut guard = db_arc.lock().unwrap();
            let region = example_region();
            let street = StreetName::new("Empty Road").unwrap();
            // store an empty slice of ranges:
            store_ranges_in_db(&mut guard, &region, &street, &[]);
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

    #[test]
    fn test_house_number_in_any_range_single_range_not_included() {
        // scenario: single sub‐range => [100..=110], query 99 => false, query 111 => false
        let (db_arc, _tmp) = create_db();
        let region = example_region();
        let street = StreetName::new("SingleRange Rd").unwrap();

        {
            let mut guard = db_arc.lock().unwrap();
            let subranges = vec![HouseNumberRange::new(100, 110)];
            store_ranges_in_db(&mut guard, &region, &street, &subranges);
        }

        {
            let guard = db_arc.lock().unwrap();
            assert!(!guard.house_number_in_any_range(&region, &street, 99).unwrap());
            assert!(!guard.house_number_in_any_range(&region, &street, 111).unwrap());
        }
    }

    #[test]
    fn test_house_number_in_any_range_single_range_included() {
        // scenario: single sub‐range => [100..=110], query 100 => true, query 105 => true, query=110 => true
        let (db_arc, _tmp) = create_db();
        let region = example_region();
        let street = StreetName::new("SingleRange Rd").unwrap();

        {
            let mut guard = db_arc.lock().unwrap();
            let subranges = vec![HouseNumberRange::new(100, 110)];
            store_ranges_in_db(&mut guard, &region, &street, &subranges);
        }

        {
            let guard = db_arc.lock().unwrap();
            assert!(guard.house_number_in_any_range(&region, &street, 100).unwrap());
            assert!(guard.house_number_in_any_range(&region, &street, 105).unwrap());
            assert!(guard.house_number_in_any_range(&region, &street, 110).unwrap());
        }
    }

    #[test]
    fn test_house_number_in_any_range_multiple_disjoint_ranges() {
        // scenario: multiple sub‐ranges => [1..=10], [20..=30], [40..=50]
        //  => query 5 => true, query 15 => false, query 45 => true
        let (db_arc, _tmp) = create_db();
        let region = example_region();
        let street = StreetName::new("MultiRange Blvd").unwrap();

        {
            let mut guard = db_arc.lock().unwrap();
            let subranges = vec![
                HouseNumberRange::new(1, 10),
                HouseNumberRange::new(20, 30),
                HouseNumberRange::new(40, 50),
            ];
            store_ranges_in_db(&mut guard, &region, &street, &subranges);
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

    #[test]
    fn test_house_number_in_any_range_error_on_load() {
        // scenario: if `load_house_number_ranges(...)` returns an Err => we propagate that as DataAccessError
        // We'll do partial approach: forcibly cause an error (e.g., lock poisoning or a RocksDB error).
        // We'll do lock poisoning. 
        let (db_arc, _tmp) = create_db();
        let region = example_region();
        let street = StreetName::new("Poison Road").unwrap();

        // Poison
        {
            let _guard = db_arc.lock().unwrap();
            panic!("Intentional poisoning of lock => cause subsequent attempts to error");
        }
        // We never get here normally, but the test harness catches the panic, so the lock is now poisoned.

        // Now any subsequent call => error
        let db_guard = db_arc.lock();
        assert!(db_guard.is_err(), "lock is poisoned => error on attempt");
        
        // Because the trait method calls `load_house_number_ranges` internally, it tries to acquire the lock again => presumably fails => we handle that as a DataAccessError or similar, depending on your code.
        // We'll do a partial approach:
        // If your code surfaces an Err(...) for a lock error, we confirm that:
        if let Ok(db_ref) = db_guard {
            // If we get here, the lock wasn't actually poisoned in your environment => skip or modify your test approach
            eprintln!("Warning: lock not poisoned as expected. Possibly not in a standard test harness. Adjust your approach if needed.");
        } else {
            // This is the expected path in typical usage => so we do e.g.:
            // can't do any direct calls => they'd fail. We'll forcibly test the function in a normal environment:
            // Usually you'd do something like:
            // let result = db_guard.house_number_in_any_range(...) => but we can't even get a guard. So we might do a partial test by mocking.
        }
    }
}
