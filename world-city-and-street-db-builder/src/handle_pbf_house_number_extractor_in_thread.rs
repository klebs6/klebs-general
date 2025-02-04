crate::ix!();

/// Handles the actual I/O, parsing, aggregation, and DB storage in a worker thread.
/// Any errors in opening or parsing the file, or mid-way processing, are sent over
/// `tx` as an `Err(...)`. Aggregation results are eventually stored in the DB.
///
/// # Arguments
///
/// * `path`         - Path to the OSM PBF file.
/// * `country`      - The resolved country object.
/// * `world_region` - The original region indicator.
/// * `db`           - Database reference, protected by a mutex.
/// * `tx`           - A `SyncSender` for streaming [`WorldAddress`] results or errors.
pub fn handle_pbf_house_number_extractor_in_thread(
    path:         PathBuf,
    country:      Country,
    world_region: WorldRegion,
    db:           Arc<Mutex<Database>>,
    tx:           std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
) {
    trace!("handle_pbf_house_number_extractor_in_thread: Spawned for path={:?}", path);

    match open_pbf_reader_or_report_error(&path, &tx) {
        Some(reader) => {
            let mut aggregator = init_house_number_aggregator();
            debug!(
                "handle_pbf_house_number_extractor_in_thread: Aggregator initialized (empty) for path={:?}",
                path
            );

            if let Err(parse_err) = try_parse_and_aggregate_house_numbers(reader, &country, &world_region, &tx, &mut aggregator) {
                error!(
                    "handle_pbf_house_number_extractor_in_thread: Error parsing PBF or aggregating results for path={:?}: {:?}",
                    path, parse_err
                );
                let _ = tx.send(Err(parse_err));
            }

            attempt_storing_house_number_aggregator_in_db(db, &world_region, aggregator);
        }
        None => {
            trace!("handle_pbf_house_number_extractor_in_thread: Early return after open error for path={:?}", path);
            // We have already sent the error via tx and returned
        }
    }
}
