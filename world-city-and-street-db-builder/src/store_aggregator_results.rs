// ---------------- [ File: src/store_aggregator_results.rs ]
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
    db:           &mut Database,
    world_region: &WorldRegion,
    aggregator:   HashMap<StreetName, Vec<HouseNumberRange>>,
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
            let region = WorldRegion::default();

            {
                let mut db_guard = db.lock().unwrap();

                let res = store_house_number_aggregator_results(&mut db_guard, &region, aggregator);
                assert!(res.is_ok());

                // Optionally load them back with load_house_number_ranges
                let loaded_opt = db_guard.load_house_number_ranges(&region, &street).unwrap();
                assert!(loaded_opt.is_some());
                let loaded = loaded_opt.unwrap();
                assert_eq!(loaded.len(), 1);
                let rng = &loaded[0];
                assert_eq!(rng.start(), &100);
                assert_eq!(rng.end(), &110);
            }
        }
    }
}
