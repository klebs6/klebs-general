// ---------------- [ File: src/validate_street_for_city.rs ]
// ---------------- [ File: src/validate_street_for_city.rs ]
crate::ix!();

/// Validates that the `[StreetName]` is present in the set of streets
/// associated with the `[CityName]` (i.e., `c_key(region, city)`).
pub fn validate_street_for_city<V:GetStreetSetForKey>(
    addr:      &WorldAddress,
    validator: &V,
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

#[cfg(test)]
mod test_validate_street_for_city {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Creates a minimal `WorldAddress` with the given region, city, and street.
    /// If your builder requires other fields, add them here.
    fn make_world_address(region: WorldRegion, city: CityName, street: StreetName) -> WorldAddress {
        WorldAddressBuilder::default()
            .region(region)
            .city(city)
            .street(street)
            // Provide a postal code if your builder requires it, or set .postal_code(None) if it's optional
            .postal_code(PostalCode::new(Country::USA, "99999").unwrap())
            .build()
            .unwrap()
    }

    /// Helper to store a set of streets under the `c_key(region, city)` used by the code.
    fn put_c_key_streets<I:StorageInterface>(
        db:      &mut I,
        region:  &WorldRegion,
        city:    &CityName,
        streets: &BTreeSet<StreetName>,
    ) {
        let key = c_key(region, city);
        let val = compress_set_to_cbor(streets);
        db.put(key, val).unwrap();
    }

    #[traced_test]
    fn test_no_street_set_for_city_returns_error() {
        let (data_access, db_arc, _temp_dir) = create_data_access::<Database>();
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("Baltimore").unwrap();
        let street = StreetName::new("North Avenue").unwrap();
        let addr = make_world_address(region, city, street);

        // We store nothing => `validator.get_street_set(&c_k)` => None
        let result = validate_street_for_city(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::CityToStreetsKeyNotFoundForCityInRegion { c_key, region, city }) => {
                // This indicates no data was found for that city in region
                assert!(
                    c_key.contains("C2S:"),
                    "Should be a c_key prefix like 'C2S:MD:baltimore'"
                );
                assert_eq!(city.name(), "baltimore");
                // Check region or other fields as needed
            }
            other => panic!("Expected CityToStreetsKeyNotFoundForCityInRegion, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_street_not_found_in_city_returns_error() {
        let (data_access, db_arc, _temp_dir) = create_data_access::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("VA").unwrap();
        let city_arlington = CityName::new("Arlington").unwrap();
        let street_wilson = StreetName::new("Wilson Blvd").unwrap();
        let street_missing = StreetName::new("NonExistent Street").unwrap();

        // Insert a set that includes `Wilson Blvd` but not `NonExistent Street`.
        let mut streets = BTreeSet::new();
        streets.insert(street_wilson.clone());
        put_c_key_streets(&mut *db_guard, &region, &city_arlington, &streets);

        // Now create an address with the missing street
        let addr = make_world_address(region, city_arlington, street_missing.clone());

        // Validate => fails
        let result = validate_street_for_city(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::StreetNotFoundForCityInRegion {
                street, city, region
            }) => {
                assert_eq!(street.name(), "nonexistent street");
                assert_eq!(city.name(), "arlington");
                assert!(matches!(region, WorldRegion::NorthAmerica(_)),
                    "Should match the region we used");
            }
            other => panic!("Expected StreetNotFoundForCityInRegion, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_street_found_in_city_returns_ok() {
        let (data_access, db_arc, _temp_dir) = create_data_access::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("DC").unwrap();
        let city_dc = CityName::new("Washington").unwrap();
        let street_pa = StreetName::new("Pennsylvania Ave").unwrap();

        // Insert a set with `Pennsylvania Ave`
        let mut streets = BTreeSet::new();
        streets.insert(street_pa.clone());
        put_c_key_streets(&mut *db_guard, &region, &city_dc, &streets);

        // Construct an address referencing that city + street
        let addr = make_world_address(region, city_dc, street_pa);
        let result = validate_street_for_city(&addr, &data_access);
        assert!(result.is_ok(), "Street is in the set => Ok(())");
    }

    #[traced_test]
    fn test_corrupted_cbor_returns_none_and_error() {
        // If the underlying set is invalid cbor => get_street_set(...) => None => same error as no data
        let (data_access, db_arc, _temp_dir) = create_data_access::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city = CityName::new("GlitchCity").unwrap();
        let street = StreetName::new("GlitchStreet").unwrap();

        let c_k = c_key(&region, &city);
        db_guard.put(c_k.clone(), b"invalid cbor").unwrap();

        let addr = make_world_address(region, city, street);
        let result = validate_street_for_city(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::CityToStreetsKeyNotFoundForCityInRegion {
                c_key, region, city
            }) => {
                // Because decode fails => None => KeyNotFound error
                assert!(c_key.contains("C2S:"));
                assert_eq!(city.name(), "glitchcity");
            }
            other => panic!("Expected CityToStreetsKeyNotFoundForCityInRegion, got {:?}", other),
        }
    }
}
