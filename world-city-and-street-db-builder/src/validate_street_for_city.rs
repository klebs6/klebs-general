// ---------------- [ File: src/validate_street_for_city.rs ]
crate::ix!();

/// Validates that the `[StreetName]` is present in the set of streets
/// associated with the `[CityName]` (i.e., `c_key(region, city)`).
pub fn validate_street_for_city(
    addr: &WorldAddress,
    validator: &DataAccess,
) -> Result<(), InvalidWorldAddress> {
    let c_k = c_key(addr.region(), addr.city());
    trace!("validate_street_for_city: using key='{}'", c_k);

    match validator.get_street_set(&c_k) {
        Some(streets) => {
            if !streets.contains(addr.street()) {
                warn!(
                    "validate_street_for_city: street='{:?}' not found for city='{}' in region={:?}",
                    addr.street(),
                    addr.city(),
                    addr.region()
                );
                return Err(InvalidWorldAddress::StreetNotFoundForCityInRegion {
                    street: addr.street().clone(),
                    city:   addr.city().clone(),
                    region: *addr.region(),
                });
            }
        }
        None => {
            warn!(
                "validate_street_for_city: no street set found for key='{}'",
                c_k
            );
            return Err(InvalidWorldAddress::CityToStreetsKeyNotFoundForCityInRegion {
                c_key: c_k,
                region: *addr.region(),
                city: addr.city().clone(),
            });
        }
    }
    Ok(())
}
