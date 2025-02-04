crate::ix!();

/// If the [`AddressRecord`] has non-empty (city, street, postcode), build a [`WorldAddress`].
/// Otherwise returns `None`. This allows skipping elements without a complete address.
pub fn build_world_address_if_possible(
    region: &WorldRegion,
    record: &AddressRecord
) -> Option<WorldAddress> {
    let (city_opt, street_opt, postcode_opt) = (record.city(), record.street(), record.postcode());

    if let (Some(city), Some(street), Some(postcode)) = (city_opt, street_opt, postcode_opt) {
        match build_world_address(region, &city, &street, &postcode) {
            Ok(addr) => {
                debug!(
                    "build_world_address_if_possible: built WorldAddress => {:?}",
                    addr
                );
                Some(addr)
            }
            Err(e) => {
                debug!("build_world_address_if_possible: failed => {:?}", e);
                None
            }
        }
    } else {
        debug!("build_world_address_if_possible: record missing city/street/postcode => skipping");
        None
    }
}
