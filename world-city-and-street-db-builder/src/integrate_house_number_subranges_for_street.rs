// ---------------- [ File: src/integrate_house_number_subranges_for_street.rs ]
crate::ix!();

/// Integrates a list of new house‐number ranges into existing DB data for the given street,
/// then writes the merged result back to the DB. Logs warnings if load/store operations fail.
///
/// # Arguments
///
/// * `db`           - Database to update.
/// * `world_region` - Region context (used in key derivation).
/// * `street`       - The street whose house‐number ranges are being updated.
/// * `new_ranges`   - A list of new [`HouseNumberRange`] values to be merged.
///
/// # Returns
///
/// * `Ok(())` on success, or if partial failures occurred but we can continue.
/// * `Err(OsmPbfParseError)` if a critical error prevents further processing.
pub fn integrate_house_number_subranges_for_street<I: StoreHouseNumberRanges + LoadExistingStreetRanges>(
    db:           &mut I,
    world_region: &WorldRegion,
    street:       &StreetName,
    new_ranges:   &Vec<HouseNumberRange>,
) -> Result<(), OsmPbfParseError> {

    trace!(
        "integrate_house_number_subranges_for_street: street='{}', merging {} new ranges",
        street,
        new_ranges.len()
    );

    // Step 1) Load existing ranges (if any)
    let existing_opt = match db.load_existing_street_ranges(world_region, street) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "integrate_house_number_subranges_for_street: could not load existing ranges for street='{}': {:?}",
                street,
                e
            );
            None
        }
    };

    // Step 2) Merge
    let merged = merge_new_subranges(existing_opt.unwrap_or_default(), new_ranges);

    // Step 3) Store
    match db.store_house_number_ranges(world_region, street, &merged) {
        Ok(_) => {
            debug!(
                "integrate_house_number_subranges_for_street: successfully stored merged ranges for street='{}'",
                street
            );
        }
        Err(e) => {
            warn!(
                "integrate_house_number_subranges_for_street: could not store updated ranges for street='{}': {:?}",
                street, e
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod test_integrate_house_number_subranges_for_street {
    use super::*;
    use tempfile::TempDir;
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::Mutex;

    /// Store house-number ranges directly into the DB.
    fn put_existing_ranges<I: StorageInterface>(
        db: &mut I,
        region: &WorldRegion,
        street: &StreetName,
        ranges: Vec<HouseNumberRange>,
    ) {
        db.store_house_number_ranges(region, street, &ranges)
          .expect("Storing initial ranges should succeed in test");
    }

    /// Read back the stored house-number ranges; return an empty vector if none found.
    fn get_stored_ranges<I: StorageInterface>(
        db: &I,
        region: &WorldRegion,
        street: &StreetName,
    ) -> Vec<HouseNumberRange> {
        match db.load_existing_street_ranges(region, street) {
            Ok(opt) => opt.unwrap_or_default(),
            Err(e) => {
                panic!("Unexpected load error in test: {:?}", e);
            }
        }
    }

    /// Shortcut for constructing a HouseNumberRange.
    fn range(start: u32, end: u32) -> HouseNumberRange {
        HouseNumberRange::new(start, end)
    }

    /// Returns a dummy region and street for testing.
    fn test_region_and_street() -> (WorldRegion, StreetName) {
        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let street = StreetName::new("Test Street").unwrap();
        (region, street)
    }

    #[traced_test]
    fn test_no_existing_ranges_new_ranges_simple() {
        let (mut db, _td) = create_temp_db::<Database>();
        let mut db_guard = db.lock().unwrap();

        let (region, street) = test_region_and_street();
        let new = vec![range(10, 15), range(20, 25)];

        integrate_house_number_subranges_for_street(&mut *db_guard, &region, &street, &new)
            .expect("Should succeed in merging new ranges into empty DB data");

        let stored = get_stored_ranges(&*db_guard, &region, &street);
        assert_eq!(stored.len(), 2, "Expected two disjoint ranges stored");
        assert_eq!(stored[0], range(10, 15));
        assert_eq!(stored[1], range(20, 25));
    }

    #[traced_test]
    fn test_existing_ranges_empty_new_ranges() {
        let (mut db, _td) = create_temp_db::<Database>();

        let mut db_guard = db.lock().unwrap();

        let (region, street) = test_region_and_street();
        let existing = vec![range(1, 5), range(10, 15)];
        let new = vec![];

        put_existing_ranges(&mut *db_guard, &region, &street, existing.clone());

        integrate_house_number_subranges_for_street(&mut *db_guard, &region, &street, &new)
            .expect("Merging empty new data should succeed");

        let stored = get_stored_ranges(&*db_guard, &region, &street);
        assert_eq!(stored, existing, "Existing data should be unchanged");
    }

    #[traced_test]
    fn test_merge_overlapping_ranges() {
        let (mut db, _td) = create_temp_db::<Database>();

        let mut db_guard = db.lock().unwrap();

        let (region, street) = test_region_and_street();

        // Existing ranges: [10..15] and [30..35]
        let existing = vec![range(10, 15), range(30, 35)];
        put_existing_ranges(&mut *db_guard, &region, &street, existing);

        // New ranges: overlapping [12..18] plus disjoint [35..40]
        let new = vec![range(12, 18), range(35, 40)];

        integrate_house_number_subranges_for_street(&mut *db_guard, &region, &street, &new)
            .expect("Should succeed in merging overlapping data");

        let stored = get_stored_ranges(&*db_guard, &region, &street);
        assert_eq!(stored.len(), 2, "Should have 2 merged ranges total");
        assert_eq!(stored[0], range(10, 18));
        assert_eq!(stored[1], range(30, 40));
    }

    #[traced_test]
    fn test_merge_completely_overlapping_new_range() {
        let (mut db, _td) = create_temp_db::<Database>();

        let mut db_guard = db.lock().unwrap();

        let (region, street) = test_region_and_street();
        let existing = vec![range(100, 110), range(120, 130)];
        put_existing_ranges(&mut *db_guard, &region, &street, existing);

        let new = vec![range(100, 130)];

        integrate_house_number_subranges_for_street(&mut *db_guard, &region, &street, &new)
            .expect("Should succeed in merging fully overlapping data");

        let stored = get_stored_ranges(&*db_guard, &region, &street);
        assert_eq!(stored.len(), 1, "Now all merges into one big range");
        assert_eq!(stored[0], range(100, 130));
    }

    #[traced_test]
    fn test_load_error_logs_warning_but_succeeds() {

        let mut failing_db = FailingLoadDatabase::new().unwrap();

        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let street = StreetName::new("Example Street").unwrap();
        let new_ranges = vec![range(1, 2)];

        let mut guard = failing_db.lock().unwrap();

        // Even though load fails, the merge should proceed.
        let result = integrate_house_number_subranges_for_street(
            &mut *guard, 
            &region, 
            &street, 
            &new_ranges
        );

        assert!(result.is_ok(), "Should not be a hard error even if load fails");
    }

    #[traced_test]
    fn test_store_error_logs_warning_but_succeeds() {

        let mut failing_db = FailingStoreDatabase::new().unwrap();

        // Insert an existing range (via the inner DB)
        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        let street = StreetName::new("Fail Street").unwrap();

        let mut guard = failing_db.lock().unwrap();

        guard.store_house_number_ranges(&region, &street, &[range(10, 15)])
            .expect("Storing to real DB works");

        // Attempt to integrate; the store error should be logged but not returned as a hard error.
        let new_ranges = vec![range(12, 18)];

        let result = integrate_house_number_subranges_for_street(
            &mut *guard, 
            &region, 
            &street, 
            &new_ranges
        );

        assert!(result.is_ok(), "Store error logs a warning, not a hard error");
    }
}
