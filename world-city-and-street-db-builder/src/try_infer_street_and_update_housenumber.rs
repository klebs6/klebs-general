crate::ix!();

/// In cases where we didn't parse an [`AddressRecord`] fully, we still might have `addr:housenumber`.
/// We attempt a partial parse for the street, and if found, update the aggregator.
pub fn try_infer_street_and_update_housenumber(
    element: &osmpbf::Element,
    country: &Country,
    aggregator: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) {
    match extract_house_number_range_from_element(element) {
        Ok(Some(range)) => {
            // Attempt to parse enough of an address record to see if there's a street
            if let Ok(record2) = AddressRecord::try_from((element, country)) {
                if let Some(street) = record2.street() {
                    debug!(
                        "try_infer_street_and_update_housenumber: storing housenumber range={:?} for street='{}'",
                        range, street
                    );
                    aggregator.entry(street.clone()).or_default().push(range);
                }
            }
        }
        Ok(None) => {
            // No housenumber => skip
        }
        Err(e) => {
            debug!("try_infer_street_and_update_housenumber: error extracting => {:?}", e);
        }
    }
}
