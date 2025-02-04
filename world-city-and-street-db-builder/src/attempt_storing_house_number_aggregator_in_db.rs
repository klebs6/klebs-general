crate::ix!();

/// Stores aggregator results into the DB, if possible. Logs warnings on failure.
/// Depending on desired behavior, you might also send an `Err` to `tx`.
pub fn attempt_storing_house_number_aggregator_in_db(
    db: Arc<Mutex<Database>>,
    world_region: &WorldRegion,
    aggregator: HashMap<StreetName, Vec<HouseNumberRange>>
) {
    trace!(
        "attempt_storing_house_number_aggregator_in_db: Storing aggregator with {} streets for region={:?}",
        aggregator.len(),
        world_region
    );

    match db.lock() {
        Ok(mut db_guard) => {
            debug!(
                "attempt_storing_house_number_aggregator_in_db: DB lock acquired; storing aggregator with {} streets",
                aggregator.len()
            );
            if let Err(e) = store_house_number_aggregator_results(&mut db_guard, world_region, aggregator) {
                warn!(
                    "attempt_storing_house_number_aggregator_in_db: Failed storing aggregator results: {:?}",
                    e
                );
            }
        }
        Err(_) => {
            warn!("attempt_storing_house_number_aggregator_in_db: Could not lock DB for region={:?}", world_region);
        }
    }
}
