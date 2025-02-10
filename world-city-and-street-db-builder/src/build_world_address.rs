// ---------------- [ File: src/build_world_address.rs ]
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

    #[traced_test]
    fn test_build_world_address_fail() {
        // Verify that creating a CityName with an empty string fails.
        let city_result = CityName::new("");
        assert!(city_result.is_err(), "Expected empty city to fail");

        // Verify that creating a StreetName with only whitespace fails.
        let street_result = StreetName::new("  ");
        assert!(street_result.is_err(), "Expected whitespace street to fail");
    }

    #[traced_test]
    fn test_build_world_address_ok() {
        let region = WorldRegion::default();
        let city = CityName::new("MyCity").unwrap();
        let street = StreetName::new("MyStreet").unwrap();
        let postal = PostalCode::new(Country::USA, "12345").unwrap();

        let wa_res = build_world_address(&region, &city, &street, &postal);
        assert!(wa_res.is_ok(), "Should succeed with normal data");
        let addr = wa_res.unwrap();
        assert_eq!(addr.region(), &region);
        assert_eq!(addr.city().name(), "mycity");
        assert_eq!(addr.street().name(), "mystreet");
        assert_eq!(addr.postal_code().code(), "12345");
    }

    #[traced_test]
    fn test_build_world_address_with_different_country() {
        // If your code uses `PostalCode::new(Country::CANADA, "H0H0H0")` or something,
        // you can test that logic. We'll do a simple example:
        let region = WorldRegion::default();
        let city = CityName::new("Montreal").unwrap();
        let street = StreetName::new("Saint Catherine").unwrap();
        let postal = PostalCode::new(Country::Canada, "H3Z2Y7").unwrap(); // example Canadian format

        let wa_res = build_world_address(&region, &city, &street, &postal);
        assert!(wa_res.is_ok(), "Canadian postal code data is valid");
        let addr = wa_res.unwrap();
        assert_eq!(addr.city().name(), "montreal");
        assert_eq!(addr.street().name(), "saint catherine");
        // check final postal code
        assert_eq!(addr.postal_code().code(), "h3z2y7");
    }

    #[traced_test]
    fn test_build_world_address_fail_on_empty_city() {
        // `CityName::new("")` => Error => can't even build `CityName`.
        let empty_city = CityName::new("");
        assert!(
            empty_city.is_err(),
            "We expect empty city => CityNameConstructionError"
        );
        // But if you forcibly create an invalid CityName via partial means,
        // you'd see `build_world_address` fails or never is called. 
        // So we typically fail earlier in city creation logic.
    }

    #[traced_test]
    fn test_build_world_address_fail_on_whitespace_street() {
        let region = WorldRegion::default();
        let city = CityName::new("ValidCity").unwrap();

        // StreetName fails => "  "
        let invalid_street = StreetName::new("  ");
        assert!(
            invalid_street.is_err(),
            "Whitespace-only street => StreetNameConstructionError"
        );
        // So we can't even proceed to call build_world_address.
    }

    #[traced_test]
    fn test_build_world_address_postal_code_failure() {
        // Suppose an empty or invalid postal code => fails creation
        let invalid_pc = PostalCode::new(Country::USA, "   ");
        assert!(invalid_pc.is_err(), "Empty or whitespace => postal code error");

        // If you forcibly create a partially valid city/street but the postal code is invalid,
        // you'd not get to build_world_address. The code basically fails earlier. 
        // If your code returns Ok(PostalCode) for odd data, adapt accordingly.
    }

    #[traced_test]
    fn test_build_world_address_very_long_names() {
        let region = WorldRegion::default();

        // Suppose your code allows city/street up to thousands of chars after normalization:
        let mut long_city_str = String::new();
        for _ in 0..500 {
            long_city_str.push_str("City"); 
        }
        // => "CityCityCity..." repeated 500 times
        let city_obj = CityName::new(&long_city_str).unwrap();

        let mut long_street_str = String::new();
        for _ in 0..300 {
            long_street_str.push_str("Street");
        }
        // => "StreetStreetStreet..." repeated 300 times
        let street_obj = StreetName::new(&long_street_str).unwrap();

        // A presumably valid postal code
        let postal = PostalCode::new(Country::USA, "99999").unwrap();

        // Now build
        let wa_res = build_world_address(&region, &city_obj, &street_obj, &postal);
        assert!(wa_res.is_ok(), "Should handle very long city/street gracefully");

        let addr = wa_res.unwrap();
        assert_eq!(addr.region(), &region);
        assert!(addr.city().name().len() > 1000, "City name is huge");
        assert!(addr.street().name().len() > 1000, "Street name is huge");
        assert_eq!(addr.postal_code().code(), "99999");
    }

    #[traced_test]
    fn test_build_world_address_placeholder_error_on_builder_fail() {
        // Suppose the builder logic might fail if something is omitted or 
        // if a partial bug triggers `map_err(...)`.
        // We'll demonstrate forcibly calling the builder with minimal data, then failing it:
        // For demonstration, let's define a scenario where the region is default 
        // and the typed objects are "somehow" invalid.

        // We'll simulate it by calling `WorldAddressBuilder` incorrectly:
        let incomplete = WorldAddressBuilder::default().build();
        assert!(incomplete.is_err(), "Uninitialized fields => builder fail");
        // That triggers .map_err(...) => your placeholder InvalidWorldAddress variant.

        // If you want to demonstrate a direct check, do:
        match incomplete {
            Err(_) => {
                // we expect your code to produce an InvalidWorldAddress variant in your map_err closure
                // but in reality, the closure won't be triggered because we haven't 
                // called build_world_address(...) yet. So this is a partial demonstration
                // of the "map_err" path in your build_world_address if the .build() fails.
            }
            Ok(_) => panic!("We expected an error from uninitialized build"),
        }
    }
}
