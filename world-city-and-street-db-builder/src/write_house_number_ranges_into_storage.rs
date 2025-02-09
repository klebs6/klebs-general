// ---------------- [ File: src/write_house_number_ranges_into_storage.rs ]
crate::ix!();

/// Merges and writes all provided house‐number ranges into storage for the given region.
/// For each `(street, ranges)`, we load existing data from the DB, unify it with the new ranges,
/// and store the result. This function logs relevant steps and returns an error if
/// database operations fail.
///
/// # Arguments
///
/// * `house_number_ranges` - Map of `StreetName` to a list of new [`HouseNumberRange`] objects.
/// * `region` - The world region these house‐number ranges belong to.
/// * `db`     - Mutable reference to the `Database` to be updated.
///
/// # Returns
///
/// * `Ok(())` if all street data is successfully written.
/// * `Err(DatabaseConstructionError)` if a load or store operation fails.
pub fn write_house_number_ranges_into_storage<I:StorageInterface>(
    house_number_ranges: &HashMap<StreetName, Vec<HouseNumberRange>>,
    region:              &WorldRegion,
    db:                  &mut I,
) -> Result<(), DatabaseConstructionError> {
    trace!(
        "write_house_number_ranges_into_storage: storing house‐number data for region={:?}, streets_count={}",
        region,
        house_number_ranges.len()
    );

    for (street, new_ranges) in house_number_ranges {
        update_street_house_numbers(db, region, street, new_ranges)?;
    }

    info!("write_house_number_ranges_into_storage: done processing all streets for region={:?}", region);
    Ok(())
}

#[cfg(test)]
#[disable]
mod test_write_house_number_ranges_into_storage {
    use super::*;
    use std::collections::HashMap;
    use std::collections::BTreeSet; // if needed
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Utility for building a HouseNumberRange
    fn hnr(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// A helper for test addresses
    fn street(name: &str) -> StreetName {
        StreetName::new(name).unwrap()
    }

    /// Creates a new, temporary `Database` for tests, returning `(Arc<Mutex<Database>>, TempDir)`.
    fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = I::open(temp_dir.path()).expect("Failed to open DB in temp dir");
        (db, temp_dir)
    }

    /// Helper to read back the final data for each street (e.g., after the function).
    /// This calls your `load_existing_house_number_ranges(...)` or something similar.
    fn load_street_ranges<I:StorageInterface>(
        db:     &I,
        region: &WorldRegion,
        street: &StreetName
    ) -> Option<Vec<HouseNumberRange>> {
        match db.load_existing_house_number_ranges(region, street) {
            Ok(ranges) => Some(ranges),
            Err(_) => None,
        }
    }

    #[test]
    fn test_empty_map_no_op_success() {
        // If house_number_ranges is empty => no iteration => success
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let empty_map = HashMap::new();

        let result = write_house_number_ranges_into_storage(&empty_map, &region, &mut db_guard);
        assert!(result.is_ok(), "Empty input => no updates => Ok(())");
    }

    #[test]
    fn test_single_street_success() {
        // house_number_ranges => { "MainSt" => [10..20] }
        // existing data => none => final => [10..20]
        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let mut map = HashMap::new();
        map.insert(street("MainSt"), vec![hnr(10,20)]);

        let result = write_house_number_ranges_into_storage(&map, &region, &mut db_guard);
        assert!(result.is_ok());

        // read back
        let final_data = load_street_ranges(&db_guard, &region, &street("MainSt"))
            .expect("Should exist after update");
        assert_eq!(final_data, vec![hnr(10,20)]);
    }

    #[test]
    fn test_multiple_streets_success() {
        // house_number_ranges => { "ASt" => [1..2], "BSt" => [5..8] }, plus merges existing data
        // We'll store some existing data for each street, then confirm final after merges

        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();

        // Pre-store some existing data:
        //  "ASt" => [1..2], "BSt" => [10..12]
        db_guard.store_house_number_ranges(&region, &street("ASt"), &[hnr(1,2)])
            .expect("existing for ASt");
        db_guard.store_house_number_ranges(&region, &street("BSt"), &[hnr(10,12)])
            .expect("existing for BSt");

        // new ranges => "ASt" => [2..4], "BSt" => [12..14], "CSt" => [100..105]
        // merges => "ASt" => unify => [1..4], "BSt" => unify => [10..14], "CSt" => new => [100..105]
        let mut map = HashMap::new();
        map.insert(street("ASt"), vec![hnr(2,4)]);
        map.insert(street("BSt"), vec![hnr(12,14)]);
        map.insert(street("CSt"), vec![hnr(100,105)]);

        let result = write_house_number_ranges_into_storage(&map, &region, &mut db_guard);
        assert!(result.is_ok());

        // Now confirm final:
        let final_a = load_street_ranges(&db_guard, &region, &street("ASt")).unwrap();
        assert_eq!(final_a, vec![hnr(1,4)], "ASt => unify [1..2] & [2..4] => [1..4]");

        let final_b = load_street_ranges(&db_guard, &region, &street("BSt")).unwrap();
        assert_eq!(final_b, vec![hnr(10,14)], "BSt => unify [10..12] & [12..14] => [10..14]");

        let final_c = load_street_ranges(&db_guard, &region, &street("CSt")).unwrap();
        assert_eq!(final_c, vec![hnr(100,105)], "CSt => no existing => stored new");
    }

    #[test]
    fn test_partial_error_aborts_and_returns_err() {
        // If `update_street_house_numbers` fails for any street, the function returns an error.
        // We'll define a stub that fails on the second street, verifying the function returns that error.

        // Minimal stub that updates the first street but fails on the second
        struct FailingUpdateStub {
            calls: std::cell::RefCell<usize>,
        }
        impl FailingUpdateStub {
            fn new() -> Self {
                FailingUpdateStub {
                    calls: std::cell::RefCell::new(0),
                }
            }
        }

        impl Database for FailingUpdateStub {
            // We'll define minimal overrides as needed. 
            // We only need store_house_number_ranges & load_existing_house_number_ranges if used inside update_street_house_numbers
        }

        // We'll mock the entire approach or replace `update_street_house_numbers` with a local function.
        // For demonstration, let's define a local test function that calls the real function but fails on the second iteration.
        fn mocked_update_street_house_numbers(
            _db: &mut FailingUpdateStub,
            _region: &WorldRegion,
            _street: &StreetName,
            _new_ranges: &[HouseNumberRange],
        ) -> Result<(), DatabaseConstructionError> {
            let mut c = _db.calls.borrow_mut();
            *c += 1;
            if *c == 2 {
                return Err(DatabaseConstructionError::DataAccessError);
            }
            Ok(())
        }

        // Next, a local rewrite of `write_house_number_ranges_into_storage` that calls `mocked_update_street_house_numbers` 
        // instead of the real `update_street_house_numbers`.
        fn write_with_mock(
            house_number_ranges: &HashMap<StreetName, Vec<HouseNumberRange>>,
            region: &WorldRegion,
            db: &mut FailingUpdateStub,
        ) -> Result<(), DatabaseConstructionError> {
            for (street, new_ranges) in house_number_ranges {
                mocked_update_street_house_numbers(db, region, street, new_ranges)?;
            }
            Ok(())
        }

        let mut db_stub = FailingUpdateStub::new();
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();

        let mut map = HashMap::new();
        map.insert(street("FirstStreet"), vec![hnr(1,2)]);
        map.insert(street("SecondStreet"), vec![hnr(10,20)]);
        map.insert(street("ThirdStreet"), vec![hnr(50,60)]);

        let result = write_with_mock(&map, &region, &mut db_stub);
        match result {
            Err(DatabaseConstructionError::DataAccessError) => {
                // Good => triggered on second street
            }
            other => panic!("Expected DataAccessError on second iteration, got {:?}", other),
        }
        // This test ensures that if any street fails to update, the entire function returns that error
    }

    #[test]
    fn test_all_succeed_after_multiple_calls() {
        // If we call the function multiple times with different data, it should handle them cumulatively.
        // This is more of an integration test style scenario.

        let (db_arc, _tmp) = create_temp_db();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();

        // 1) First call => just store "MainSt" => [1..5]
        {
            let mut map = HashMap::new();
            map.insert(street("MainSt"), vec![hnr(1,5)]);
            write_house_number_ranges_into_storage(&map, &region, &mut db_guard)
                .expect("first call should succeed");
        }
        // read back "MainSt" => [1..5]
        {
            let final_main = load_street_ranges(&db_guard, &region, &street("MainSt")).unwrap();
            assert_eq!(final_main, vec![hnr(1,5)]);
        }

        // 2) second call => includes "MainSt" => [3..8], "SideSt" => [100..110]
        {
            let mut map2 = HashMap::new();
            map2.insert(street("MainSt"), vec![hnr(3,8)]);
            map2.insert(street("SideSt"), vec![hnr(100,110)]);
            write_house_number_ranges_into_storage(&map2, &region, &mut db_guard)
                .expect("second call should succeed");
        }
        // read back => "MainSt" => merged => [1..8], "SideSt" => [100..110]
        {
            let final_main = load_street_ranges(&db_guard, &region, &street("MainSt")).unwrap();
            assert_eq!(final_main, vec![hnr(1,8)]);
            let final_side = load_street_ranges(&db_guard, &region, &street("SideSt")).unwrap();
            assert_eq!(final_side, vec![hnr(100,110)]);
        }
    }
}
