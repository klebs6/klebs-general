// ---------------- [ File: src/parse_osm_pbf_and_build_house_number_ranges.rs ]
crate::ix!();

/// Loads an OSM PBF file, extracting all [`AddressRecord`]s and accumulating
/// [`HouseNumberRange`] objects in memory. This function is suitable for smaller
/// to medium‐sized data sets that fit into RAM.
///
/// **If the data is massive**, consider a streaming approach where intermediate
/// results are regularly flushed to disk or a database instead of being stored
/// in a large in‐memory map.
///
/// # Arguments
///
/// * `path`   - Filesystem path to a `.pbf` file containing OSM data.
/// * `region` - A [`WorldRegion`] from which we infer the `Country`.
///
/// # Returns
///
/// * `Ok((Vec<AddressRecord>, HashMap<StreetName, Vec<HouseNumberRange>>))`:
///   A list of addresses and a map from street names to collected house‐number ranges.
/// * `Err(OsmPbfParseError)` if reading or parsing the file fails.
pub fn load_osm_data_with_housenumbers(
    path: impl AsRef<Path>,
    region: &WorldRegion,
) -> Result<(Vec<AddressRecord>, HashMap<StreetName, Vec<HouseNumberRange>>), OsmPbfParseError> {
    trace!(
        "load_osm_data_with_housenumbers: start path={:?}, region={:?}",
        path.as_ref(),
        region
    );

    // Step 1: Infer the Country from the given region.
    let country = infer_country_from_region(region)?;

    // Step 2: Open the OSM PBF file for reading.
    let reader = open_osm_pbf_reader(&path)?;

    // Step 3: We’ll accumulate addresses and house‐number ranges in memory.
    let mut street_hnr_map: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();
    let mut addresses = Vec::new();

    // Step 4: Process the PBF file’s elements in a single pass.
    collect_address_and_housenumber_data(reader, &country, &mut addresses, &mut street_hnr_map)?;

    info!(
        "load_osm_data_with_housenumbers: completed. Found {} addresses; {} streets with house‐number data",
        addresses.len(),
        street_hnr_map.len()
    );

    Ok((addresses, street_hnr_map))
}
