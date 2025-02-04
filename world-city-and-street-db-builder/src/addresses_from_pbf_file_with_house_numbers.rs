// ---------------- [ File: src/addresses_from_pbf_file_with_house_numbers.rs ]
crate::ix!();

/// The top-level function orchestrates:
/// 1) Converting a [`WorldRegion`] into a [`Country`].
/// 2) Creating a streaming channel.
/// 3) Spawning a background thread to:
///    - Open and parse the OSM PBF file.
///    - Accumulate house number ranges in memory.
///    - Send intermediate address results over the channel.
///    - Store aggregated house number ranges into the database.
/// 4) Returning the consumer side of that channel as an [`Iterator`].
///
/// # Arguments
///
/// * `path`         - Path to an OSM PBF file on disk.
/// * `world_region` - Geographic region used for country inference.
/// * `db`           - Shared mutable database handle.
///
/// # Returns
///
/// * `Ok(impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>)` on success.
/// * `Err(OsmPbfParseError)` if the country conversion fails immediately.
pub fn addresses_from_pbf_file_with_house_numbers(
    path: PathBuf,
    world_region: WorldRegion,
    db: Arc<Mutex<Database>>,
) -> Result<impl Iterator<Item = Result<WorldAddress, OsmPbfParseError>>, OsmPbfParseError> {
    trace!("addresses_from_pbf_file_with_house_numbers: Invoked with path={:?}, region={:?}", path, world_region);

    let country = try_resolve_country(world_region)?;
    trace!("addresses_from_pbf_file_with_house_numbers: Resolved country={:?}", country);

    let (tx, rx) = create_address_stream_channel();
    trace!("addresses_from_pbf_file_with_house_numbers: Created sync_channel for address streaming");

    // Move ownership into background thread
    thread::spawn(move || {
        handle_pbf_house_number_extractor_in_thread(path, country, world_region, db, tx);
    });

    // Provide the consumer an iterator of results
    Ok(rx.into_iter())
}

/// Tries to convert the provided [`WorldRegion`] into a [`Country`].
/// Returns an [`OsmPbfParseError`] if the conversion is invalid.
///
/// This is a thin wrapper over `Country::try_from(...)` with extra tracing.
fn try_resolve_country(
    region: WorldRegion
) -> Result<Country, OsmPbfParseError> {
    trace!("try_resolve_country: Attempting to convert region={:?}", region);
    let country = Country::try_from(region)?;
    debug!("try_resolve_country: Successfully resolved to {:?}", country);
    Ok(country)
}

/// Creates a bounded sync channel for streaming address results.
/// Returns `(SyncSender, Receiver)`.
fn create_address_stream_channel(
) -> (
    std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    std::sync::mpsc::Receiver<Result<WorldAddress, OsmPbfParseError>>
) {
    // Capacity of 1000 is arbitrary; can be tweaked depending on performance needs.
    std::sync::mpsc::sync_channel(1000)
}

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
fn handle_pbf_house_number_extractor_in_thread(
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

/// Helper that attempts to open the OSM PBF file. If successful, returns the reader.
/// On failure, sends the error through `tx` and returns `None`.
fn open_pbf_reader_or_report_error(
    path: &PathBuf,
    tx: &std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
) -> Option<osmpbf::ElementReader<std::io::BufReader<std::fs::File>>> {
    trace!("open_pbf_reader_or_report_error: Opening OSM PBF at {:?}", path);

    match open_osm_pbf_reader(path) {
        Ok(reader) => {
            debug!("open_pbf_reader_or_report_error: Successfully opened {:?}", path);
            Some(reader)
        }
        Err(e) => {
            error!("open_pbf_reader_or_report_error: Failed to open {:?}: {:?}", path, e);
            let _ = tx.send(Err(e));
            None
        }
    }
}

/// Initializes the aggregator for house number ranges. Currently just a new HashMap.
fn init_house_number_aggregator() -> HashMap<StreetName, Vec<HouseNumberRange>> {
    trace!("init_house_number_aggregator: Creating new aggregator");
    HashMap::new()
}

/// Attempts to parse the OSM PBF data, populate `aggregator`, and stream out addresses.
/// Returns an error if parsing fails or an error is encountered mid-processing.
fn try_parse_and_aggregate_house_numbers<R:Read+Send+Sync>(
    reader: osmpbf::ElementReader<R>,
    country: &Country,
    world_region: &WorldRegion,
    tx: &std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    aggregator: &mut HashMap<StreetName, Vec<HouseNumberRange>>
) -> Result<(), OsmPbfParseError> {
    trace!(
        "try_parse_and_aggregate_house_numbers: Parsing OSM from reader with region={:?}, country={:?}",
        world_region, country
    );

    parse_and_aggregate_osm(
        reader,
        country,
        world_region,
        tx,
        aggregator
    )
}

/// Stores aggregator results into the DB, if possible. Logs warnings on failure.
/// Depending on desired behavior, you might also send an `Err` to `tx`.
fn attempt_storing_house_number_aggregator_in_db(
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
