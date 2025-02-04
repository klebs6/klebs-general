// ---------------- [ File: src/build_world_address.rs ]
crate::ix!();

/// Helper that builds a `WorldAddress` from region, city, street, postal.
/// Returns an error if the builder fails.
pub fn build_world_address(
    region: &WorldRegion,
    city: &CityName,
    street: &StreetName,
    postal_code: &crate::PostalCode,
) -> Result<WorldAddress, InvalidWorldAddress>
{
    trace!("build_world_address: city={}, street={}, postal={}", city.name(), street.name(), postal_code.code());
    // For brevity, we assume the builder might only fail on unusual data
    // (your real code might store city/region in the error).
    WorldAddressBuilder::default()
        .region(*region)
        .city(city.clone())
        .street(street.clone())
        .postal_code(postal_code.clone())
        .build()
        .map_err(|_| {
            // Construct any suitable `InvalidWorldAddress` variant or a custom error.
            // For demonstration, we just do a placeholder:
            InvalidWorldAddress::CityNotFoundForPostalCodeInRegion {
                city: city.clone(),
                postal_code: postal_code.clone(),
                region: *region,
            }
        })
}

#[cfg(test)]
mod build_world_address_tests {
    use super::*;

    #[test]
    fn test_build_world_address_ok() {
        let region = WorldRegion::default();
        let city = CityName::new("MyCity").unwrap();
        let street = StreetName::new("MyStreet").unwrap();
        let postal = PostalCode::new(Country::USA, "12345").unwrap();

        let wa = build_world_address(&region, &city, &street, &postal);
        assert!(wa.is_ok());
        let addr = wa.unwrap();
        assert_eq!(addr.city().name(), "mycity");
        assert_eq!(addr.street().name(), "mystreet");
        assert_eq!(addr.postal_code().code(), "12345");
    }

    #[test]
    fn test_build_world_address_fail() {
        // Verify that creating a CityName with an empty string fails.
        let city_result = CityName::new("");
        assert!(city_result.is_err(), "Expected empty city to fail");

        // Verify that creating a StreetName with only whitespace fails.
        let street_result = StreetName::new("  ");
        assert!(street_result.is_err(), "Expected whitespace street to fail");
    }
}
