crate::ix!();

/// Validates that the `[CityName]` is present in the set of cities associated
/// with the `[PostalCode]` (i.e., `z2c_key(region, postal_code)`).
pub fn validate_city_for_postal_code(
    addr: &WorldAddress,
    validator: &DataAccess,
) -> Result<(), InvalidWorldAddress> {
    let z2c_k = z2c_key(&addr.region, &addr.postal_code);
    trace!("validate_city_for_postal_code: using key='{}'", z2c_k);

    match validator.get_city_set(&z2c_k) {
        Some(city_set) => {
            if !city_set.contains(addr.city()) {
                warn!(
                    "validate_city_for_postal_code: city='{:?}' not found for postal_code='{:?}' in region={:?}",
                    addr.city(),
                    addr.postal_code(),
                    addr.region()
                );
                return Err(InvalidWorldAddress::CityNotFoundForPostalCodeInRegion {
                    city: addr.city().clone(),
                    postal_code: addr.postal_code().clone(),
                    region: *addr.region(),
                });
            }
        }
        None => {
            warn!(
                "validate_city_for_postal_code: no city set found for key='{}'",
                z2c_k
            );
            return Err(InvalidWorldAddress::PostalCodeToCityKeyNotFoundForRegion {
                z2c_key: z2c_k,
                region: *addr.region(),
                postal_code: addr.postal_code().clone(),
            });
        }
    }
    Ok(())
}
