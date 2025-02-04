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
    let reader = open_osm_pbf_reader(path.as_ref())?;

    // Step 3: We’ll accumulate addresses and house‐number ranges in memory.
    let mut street_hnr_map: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();
    let mut addresses = Vec::new();

    // Step 4: Process the PBF file’s elements in a single pass.
    collect_address_and_housenumber_data(&reader, &country, &mut addresses, &mut street_hnr_map)?;

    info!(
        "load_osm_data_with_housenumbers: completed. Found {} addresses; {} streets with house‐number data",
        addresses.len(),
        street_hnr_map.len()
    );

    Ok((addresses, street_hnr_map))
}

/// Converts a [`WorldRegion`] into a [`Country`], logging the attempt and result.
/// Returns an error if the region is unknown to our system.
fn infer_country_from_region(
    region: &WorldRegion
) -> Result<Country, OsmPbfParseError> {
    trace!("infer_country_from_region: region={:?}", region);
    let country = Country::try_from(*region)?;
    debug!("infer_country_from_region: resolved to {:?}", country);
    Ok(country)
}

/// Opens an OSM PBF file using `osmpbf::ElementReader::from_path(...)`,
/// returning any file I/O or parser errors.
fn open_osm_pbf_reader(path: &Path) -> Result<osmpbf::ElementReader, OsmPbfParseError> {
    trace!("open_osm_pbf_reader: attempting to open {:?}", path);
    let reader = osmpbf::ElementReader::from_path(path)?;
    debug!("open_osm_pbf_reader: successfully opened {:?}", path);
    Ok(reader)
}

/// Iterates through all OSM elements in the file, extracting both addresses
/// and house‐number ranges. The results are appended to `addresses` and
/// `street_hnr_map`.
fn collect_address_and_housenumber_data(
    reader: &osmpbf::ElementReader,
    country: &Country,
    addresses: &mut Vec<AddressRecord>,
    street_hnr_map: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) -> Result<(), OsmPbfParseError> {
    trace!("collect_address_and_housenumber_data: starting iteration");

    let mut count = 0usize;
    reader.for_each(|element| {
        process_single_osm_element(element, country, addresses, street_hnr_map)?;
        count += 1;

        // Periodic log to observe progress
        if count % 100_000 == 0 {
            info!(
                "collect_address_and_housenumber_data: processed {} elements so far...",
                count
            );
        }
        Ok(())
    })?;

    debug!("collect_address_and_housenumber_data: complete. total elements={}", count);
    Ok(())
}

/// For one OSM element, we:
///   1. Attempt to parse an [`AddressRecord`] via `AddressRecord::try_from(...)`.
///   2. Extract a [`HouseNumberRange`] if present.
///   3. If both a street name and house‐number range exist, store them in `street_hnr_map`.
fn process_single_osm_element(
    element: &osmpbf::Element,
    country: &Country,
    addresses: &mut Vec<AddressRecord>,
    street_hnr_map: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) -> Result<(), OsmPbfParseError> {
    trace!("process_single_osm_element: analyzing an OSM element");

    // Attempt an AddressRecord
    let record_result = AddressRecord::try_from((element, country));
    if let Ok(addr) = &record_result {
        debug!("process_single_osm_element: got AddressRecord => pushing to addresses");
        addresses.push(addr.clone());
    }

    // Attempt a HouseNumberRange
    let hnr_result = extract_house_number_range_from_element(element);

    // If we found a HNR and we have a valid street, record it
    if let Ok(Some(hnr)) = hnr_result {
        let maybe_street = match &record_result {
            Ok(addr) => addr.street(),  // If we already have an AddressRecord
            Err(_) => {
                // If the AddressRecord parse failed, we can try again or skip
                // Try again just to see if there's a street
                if let Ok(addr2) = AddressRecord::try_from((element, country)) {
                    addr2.street()
                } else {
                    None
                }
            }
        };

        if let Some(street) = maybe_street {
            debug!(
                "process_single_osm_element: found HNR={:?} => adding to street='{}'",
                hnr,
                street
            );
            street_hnr_map.entry(street.clone()).or_default().push(hnr);
        }
    }

    Ok(())
}
