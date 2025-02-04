crate::ix!();

/// For one OSM element, we:
///   1. Attempt to parse an [`AddressRecord`] via `AddressRecord::try_from(...)`.
///   2. Extract a [`HouseNumberRange`] if present.
///   3. If both a street name and house‐number range exist, store them in `street_hnr_map`.
pub fn process_single_osm_element(
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
