// ---------------- [ File: src/validate_city_for_postal_code.rs ]
crate::ix!();

/// Validates that the `[CityName]` is present in the set of cities associated
/// with the `[PostalCode]` (i.e., `z2c_key(region, postal_code)`).
pub fn validate_city_for_postal_code<V:GetCitySetForKey>(
    addr:      &WorldAddress,
    validator: &V,
) -> Result<(), InvalidWorldAddress> {
    let z2c_k = z2c_key(addr.region(), addr.postal_code());
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

#[cfg(test)]
mod test_validate_city_for_postal_code {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Creates a minimal `WorldAddress` with given region, city, and postal code.
    fn make_world_address(region: WorldRegion, city: CityName, postal_code: PostalCode) -> WorldAddress {
        WorldAddressBuilder::default()
            .region(region)
            .city(city)
            .postal_code(postal_code)
            // street is not used here, but if your builder requires it, provide a stub
            .street(StreetName::new("example street").unwrap())
            .build()
            .unwrap()
    }

    /// Helper that stores a set of cities under `z2c_key(region, postal_code)`.
    fn put_z2c_data<I:StorageInterface>(
        db:          &mut I,
        region:      &WorldRegion,
        postal_code: &PostalCode,
        cities:      &BTreeSet<CityName>,
    ) {
        let key = z2c_key(region, postal_code);
        let val = compress_set_to_cbor(cities);
        db.put(key, val).unwrap();
    }

    #[traced_test]
    fn test_no_city_set_exists_returns_error() {

        let (data_access, db_arc, _temp_dir) = create_data_access::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region      = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city        = CityName::new("Baltimore").unwrap();
        let postal_code = PostalCode::new(Country::USA, "21201").unwrap();
        let addr        = make_world_address(region, city, postal_code);
        drop(db_guard);

        // We intentionally do not store anything under z2c => so get_city_set => None
        let result = validate_city_for_postal_code(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::PostalCodeToCityKeyNotFoundForRegion { z2c_key, region, postal_code }) => {
                // Ok => verified
                assert!(z2c_key.contains("Z2C:"), "Key should be a 'Z2C:...' prefix");
            }
            other => panic!("Expected PostalCodeToCityKeyNotFoundForRegion, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_city_not_found_in_set_returns_error() {
        let (data_access, db_arc, _temp_dir) = create_data_access::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region         = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city_baltimore = CityName::new("Baltimore").unwrap();
        let city_annapolis = CityName::new("Annapolis").unwrap();
        let postal_code    = PostalCode::new(Country::USA, "21201").unwrap();
        let addr           = make_world_address(region, city_baltimore.clone(), postal_code.clone());

        // We'll store a set that contains [Annapolis], but not Baltimore
        let mut city_set = BTreeSet::new();
        city_set.insert(city_annapolis);
        put_z2c_data(&mut *db_guard, &addr.region(), &addr.postal_code(), &city_set);
        drop(db_guard);

        let result = validate_city_for_postal_code(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::CityNotFoundForPostalCodeInRegion {
                city, postal_code, region
            }) => {
                assert_eq!(city.name(), "baltimore");
                assert_eq!(postal_code.code(), "21201");
                assert!(matches!(region, WorldRegion::NorthAmerica(_)), "Region is the same as the address region");
            }
            other => panic!("Expected CityNotFoundForPostalCodeInRegion, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_city_found_in_set_returns_ok() {
        let (data_access, db_arc, _temp_dir) = create_data_access::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region         = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city_baltimore = CityName::new("Baltimore").unwrap();
        let postal_code    = PostalCode::new(Country::USA, "21201").unwrap();
        let addr           = make_world_address(region, city_baltimore.clone(), postal_code.clone());

        // Store a set that DOES contain Baltimore
        let mut city_set = BTreeSet::new();
        city_set.insert(city_baltimore.clone());
        put_z2c_data(&mut *db_guard, &addr.region(), &addr.postal_code(), &city_set);
        drop(db_guard);

        let result = validate_city_for_postal_code(&addr, &data_access);
        assert!(result.is_ok(), "City is in the set => Ok(())");
    }

    #[traced_test]
    fn test_corrupted_data_returns_none_and_error() {
        // If the z2c data is invalid CBOR, get_city_set => None => 
        // => we produce PostalCodeToCityKeyNotFoundForRegion
        let (data_access, db_arc, _temp_dir) = create_data_access::<Database>();
        let mut db_guard = db_arc.lock().unwrap();

        let region      = WorldRegion::try_from_abbreviation("MD").unwrap();
        let city        = CityName::new("CorruptCity").unwrap();
        let postal_code = PostalCode::new(Country::USA, "99999").unwrap();
        let addr        = make_world_address(region, city, postal_code);

        // Insert invalid data
        let z2c_k = z2c_key(&addr.region(), &addr.postal_code());
        db_guard.put(z2c_k.clone(), b"not valid cbor").unwrap();
        drop(db_guard);

        let result = validate_city_for_postal_code(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::PostalCodeToCityKeyNotFoundForRegion {
                z2c_key, region, postal_code
            }) => {
                // Because the code sees decode fail => None
                assert!(z2c_key.contains("Z2C:"));
                assert_eq!(postal_code.code(), "99999");
            }
            other => panic!("Expected PostalCodeToCityKeyNotFoundForRegion from corrupted data, got {:?}", other),
        }
    }
}
