// ---------------- [ File: src/validate_street_for_city.rs ]
crate::ix!();

pub fn validate_street_for_city<V>(
    addr: &WorldAddress,
    validator: &V,
) -> Result<(), InvalidWorldAddress>
where
    V: GetStreetSetForKey
    + CityNamesForPostalCodeInRegion 
    + StreetNamesForPostalCodeInRegion
    + PostalCodesForCityInRegion
    + PostalCodesForStreetInRegion,
{
    let region = addr.region();
    let city_key = c_key(region, addr.city());
    trace!(
        "validate_street_for_city: region={:?}, city={}, street={}, zip={}",
        region,
        addr.city().name(),
        addr.street().name(),
        addr.postal_code().code()
    );

    // 1) Direct check: see if city->streets includes this street
    match validator.get_street_set(&city_key) {
        Some(street_set) => {
            if street_set.contains(addr.street()) {
                info!(
                    "validate_street_for_city: direct match found: street='{}' is in city='{}'",
                    addr.street().name(),
                    addr.city().name()
                );
                return Ok(());
            } else {
                warn!(
                    "validate_street_for_city: street='{}' not found in city='{}' => attempting fallback check",
                    addr.street().name(),
                    addr.city().name(),
                );

                // 2) Fallback: city->zips, street->zips => if they share address’s zip => success
                if let Some(city_zips) = validator.postal_codes_for_city_in_region(region, addr.city()) {
                    info!("fallback: city='{}' => city_zips={:#?}", addr.city().name(), city_zips);

                    if !city_zips.contains(addr.postal_code()) {
                        warn!(
                            "fallback: city='{}' does NOT contain zip='{}'; cannot validate street='{}'",
                            addr.city().name(),
                            addr.postal_code().code(),
                            addr.street().name()
                        );
                        return Err(InvalidWorldAddress::StreetNotFoundForCityInRegion {
                            street: addr.street().clone(),
                            city: addr.city().clone(),
                            region: *region,
                        });
                    }
                } else {
                    warn!(
                        "fallback: no city_zips found for city='{}' => fallback fails",
                        addr.city().name()
                    );
                    return Err(InvalidWorldAddress::StreetNotFoundForCityInRegion {
                        street: addr.street().clone(),
                        city: addr.city().clone(),
                        region: *region,
                    });
                }

                if let Some(street_zips) = validator.postal_codes_for_street_in_region(region, addr.street()) {
                    info!("fallback: street='{}' => street_zips={:#?}", addr.street().name(), street_zips);

                    if street_zips.contains(addr.postal_code()) {
                        info!(
                            "fallback: city='{}' and street='{}' share zip='{}' => fallback success",
                            addr.city().name(),
                            addr.street().name(),
                            addr.postal_code().code()
                        );
                        return Ok(());
                    } else {
                        warn!(
                            "fallback: street='{}' does NOT contain zip='{}'; cannot validate",
                            addr.street().name(),
                            addr.postal_code().code()
                        );
                        return Err(InvalidWorldAddress::StreetNotFoundForCityInRegion {
                            street: addr.street().clone(),
                            city: addr.city().clone(),
                            region: *region,
                        });
                    }
                } else {
                    warn!(
                        "fallback: no street_zips found for street='{}' => fallback fails",
                        addr.street().name()
                    );
                    return Err(InvalidWorldAddress::StreetNotFoundForCityInRegion {
                        street: addr.street().clone(),
                        city: addr.city().clone(),
                        region: *region,
                    });
                }
            }
        }
        None => {
            warn!(
                "validate_street_for_city: no city->streets data for city='{}' => error",
                addr.city().name()
            );
            return Err(InvalidWorldAddress::CityToStreetsKeyNotFoundForCityInRegion {
                c_key: city_key,
                region: *region,
                city: addr.city().clone(),
            });
        }
    }
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
        drop(db_guard);

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
        drop(db_guard);

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
        drop(db_guard);

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
