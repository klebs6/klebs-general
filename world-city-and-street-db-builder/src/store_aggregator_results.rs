// ---------------- [ File: src/store_house_number_aggregator_results.rs ]
crate::ix!();

/// Takes the aggregator (`street -> Vec<HouseNumberRange>`) and merges each entry
/// with existing data in the database, storing the final sets back. Logs warnings
/// on failures, but continues processing.
///
/// # Arguments
///
/// * `db`           - A mutable reference to the database.
/// * `world_region` - The region scoping these house‐number entries.
/// * `aggregator`   - A map from `StreetName` to a list of new [`HouseNumberRange`] objects.
///
/// # Returns
///
/// * `Ok(())` if all aggregator data is processed successfully (warnings may still occur).
/// * `Err(OsmPbfParseError)` if a critical error arises (e.g., DB I/O error).
pub fn store_house_number_aggregator_results(
    db: &mut Database,
    world_region: &WorldRegion,
    aggregator: HashMap<StreetName, Vec<HouseNumberRange>>,
) -> Result<(), OsmPbfParseError> {
    trace!(
        "store_house_number_aggregator_results: storing data for {} streets in region={:?}",
        aggregator.len(),
        world_region
    );

    for (street, subranges) in aggregator {
        integrate_house_number_subranges_for_street(db, world_region, &street, subranges)?;
    }

    info!("store_house_number_aggregator_results: All aggregator data processed.");
    Ok(())
}

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
fn integrate_house_number_subranges_for_street(
    db: &mut Database,
    world_region: &WorldRegion,
    street: &StreetName,
    new_ranges: Vec<HouseNumberRange>,
) -> Result<(), OsmPbfParseError> {
    trace!(
        "integrate_house_number_subranges_for_street: street='{}', merging {} new ranges",
        street,
        new_ranges.len()
    );

    // Step 1) Load existing ranges (if any)
    let existing_opt = match load_existing_street_ranges(db, world_region, street) {
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
    match store_merged_street_ranges(db, world_region, street, &merged) {
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

/// Loads existing house‐number ranges for the specified street from the DB.
fn load_existing_street_ranges(
    db: &mut Database,
    world_region: &WorldRegion,
    street: &StreetName,
) -> Result<Option<Vec<HouseNumberRange>>, OsmPbfParseError> {
    trace!(
        "load_existing_street_ranges: loading for street='{}' in region={:?}",
        street,
        world_region
    );
    let existing = load_house_number_ranges(db, world_region, street)?;
    Ok(existing)
}

/// Merges newly extracted subranges into the existing list, returning a consolidated list.
/// This example calls an existing helper (like `merge_house_number_range`) for each range.
///
/// # Returns
///
/// * A new `Vec<HouseNumberRange>` containing the merged results.
fn merge_new_subranges(
    mut current: Vec<HouseNumberRange>,
    new_ranges: Vec<HouseNumberRange>,
) -> Vec<HouseNumberRange> {
    trace!(
        "merge_new_subranges: current={} existing ranges, new={} ranges",
        current.len(),
        new_ranges.len()
    );

    for rng in new_ranges {
        current = merge_house_number_range(current, rng);
    }
    current
}

/// Stores the merged house‐number ranges back into the DB for the given street.
fn store_merged_street_ranges(
    db: &mut Database,
    world_region: &WorldRegion,
    street: &StreetName,
    merged: &[HouseNumberRange],
) -> Result<(), OsmPbfParseError> {
    trace!(
        "store_merged_street_ranges: storing {} merged ranges for street='{}'",
        merged.len(),
        street
    );

    store_house_number_ranges(db, world_region, street, merged)?;
    Ok(())
}

#[cfg(test)]
mod store_aggregator_results_tests {
    use super::*;

    #[test]
    fn test_store_aggregator_results_empty() {
        let mut aggregator = HashMap::new();
        let tmp_dir = TempDir::new().unwrap();
        let db = Database::open(tmp_dir.path()).unwrap();
        let mut db_guard = db.lock().unwrap();

        let region = WorldRegion::default();

        // storing an empty aggregator => no effect
        let res = store_house_number_aggregator_results(&mut db_guard, &region, aggregator);
        assert!(res.is_ok());
    }

    #[test]
    fn test_store_aggregator_results_single_street() {
        // aggregator => "north avenue" => [ HouseNumberRange(100..=110) ]
        let mut aggregator = HashMap::new();
        let street = StreetName::new("North Avenue").unwrap();
        aggregator.insert(street.clone(), vec![HouseNumberRange::new(100, 110)]);

        let tmp_dir = TempDir::new().unwrap();
        let db = Database::open(tmp_dir.path()).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            let region = WorldRegion::default();

            let res = store_house_number_aggregator_results(&mut db_guard, &region, aggregator);
            assert!(res.is_ok());

            // Optionally load them back with load_house_number_ranges
            let loaded_opt = load_house_number_ranges(&db_guard, &region, &street).unwrap();
            assert!(loaded_opt.is_some());
            let loaded = loaded_opt.unwrap();
            assert_eq!(loaded.len(), 1);
            let rng = &loaded[0];
            assert_eq!(rng.start(), &100);
            assert_eq!(rng.end(), &110);
        }
    }
}
