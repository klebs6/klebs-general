crate::ix!();

/// Validates that the `[StreetName]` is present in the set of streets
/// associated with the `[PostalCode]` (i.e., `s_key(region, postal_code)`).
pub fn validate_street_for_postal_code(
    addr: &WorldAddress,
    validator: &DataAccess,
) -> Result<(), InvalidWorldAddress> {
    let s_k = s_key(&addr.region, &addr.postal_code);
    trace!("validate_street_for_postal_code: using key='{}'", s_k);

    match validator.get_street_set(&s_k) {
        Some(streets) => {
            if !streets.contains(addr.street()) {
                warn!(
                    "validate_street_for_postal_code: street='{:?}' not found for postal_code='{:?}' in region={:?}",
                    addr.street(),
                    addr.postal_code(),
                    addr.region()
                );
                return Err(InvalidWorldAddress::StreetNotFoundForPostalCodeInRegion {
                    street: addr.street().clone(),
                    postal_code: addr.postal_code().clone(),
                    region: *addr.region(),
                });
            }
        }
        None => {
            warn!(
                "validate_street_for_postal_code: no street set found for key='{}'",
                s_k
            );
            return Err(InvalidWorldAddress::PostalCodeToStreetKeyNotFoundForRegion {
                s_key: s_k,
                region: *addr.region(),
                postal_code: addr.postal_code().clone(),
            });
        }
    }
    Ok(())
}
