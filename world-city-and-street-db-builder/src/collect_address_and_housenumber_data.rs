crate::ix!();

/// Iterates through all OSM elements in the file, extracting both addresses
/// and house‐number ranges. The results are appended to `addresses` and
/// `street_hnr_map`.
pub fn collect_address_and_housenumber_data(
    reader: &osmpbf::ElementReader<std::io::BufReader<std::fs::File>>,
    country: &Country,
    addresses: &mut Vec<AddressRecord>,
    street_hnr_map: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) -> Result<(), OsmPbfParseError> {
    trace!("collect_address_and_housenumber_data: starting iteration");

    let mut count = 0usize;
    reader.for_each(|element| {
        process_single_osm_element(&element, country, addresses, street_hnr_map)
            .expect("could not process single osm element");
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
