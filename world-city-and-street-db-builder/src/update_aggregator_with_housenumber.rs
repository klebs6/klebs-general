crate::ix!();

/// Extracts a [`HouseNumberRange`] (if any) from the element and, if found,
/// updates the aggregator entry for the element's street (taken from `record`).
pub fn update_aggregator_with_housenumber(
    element: &osmpbf::Element,
    record: &AddressRecord,
    aggregator: &mut HashMap<StreetName, Vec<HouseNumberRange>>,
) {
    match extract_house_number_range_from_element(element) {
        Ok(Some(range)) => {
            if let Some(street) = record.street() {
                trace!("update_aggregator_with_housenumber: found housenumber range={:?}, street={}", range, street);
                aggregator.entry(street.clone()).or_default().push(range);
            }
        }
        Ok(None) => {
            // No house-number => do nothing
        }
        Err(e) => {
            debug!("update_aggregator_with_housenumber: error extracting house number => {:?}", e);
        }
    }
}
