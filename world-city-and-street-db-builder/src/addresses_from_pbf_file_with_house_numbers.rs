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
