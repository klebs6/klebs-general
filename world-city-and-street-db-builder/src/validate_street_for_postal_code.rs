// ---------------- [ File: src/validate_street_for_postal_code.rs ]
crate::ix!();

/// Validates that the `[StreetName]` is present in the set of streets
/// associated with the `[PostalCode]` (i.e., `s_key(region, postal_code)`).
pub fn validate_street_for_postal_code<V:GetStreetSetForKey>(
    addr:      &WorldAddress,
    validator: &V,
) -> Result<(), InvalidWorldAddress> {
    let s_k = s_key(addr.region(), addr.postal_code());
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

#[cfg(test)]
#[disable]
mod test_validate_street_for_postal_code {
    use super::*;
    use std::collections::BTreeSet;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Creates a minimal `WorldAddress` with region/postal_code/street for validation.
    /// If your builder requires more fields (city, etc.), add them here.
    fn make_world_address(
        region: WorldRegion,
        postal_code: PostalCode,
        street: StreetName
    ) -> WorldAddress {
        WorldAddressBuilder::default()
            .region(region)
            .postal_code(postal_code)
            .street(street)
            // Provide city if your builder needs it, or set to None if it's optional
            .city(CityName::new("dummycity").unwrap())
            .build()
            .unwrap()
    }

    /// Creates a DataAccess from a newly opened DB in a temp directory.
    fn create_data_access<I:StorageInterface>() -> (DataAccess<I>, Arc<Mutex<I>>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db = I::open(temp_dir.path()).expect("Failed to open DB");
        let db_arc = db.clone();
        let data_access = DataAccess::with_db(db);
        (data_access, db_arc, temp_dir)
    }

    /// Inserts a set of streets under the `s_key(region, postal_code)`.
    fn put_s_key_streets<I:StorageInterface>(
        db:          &mut I,
        region:      &WorldRegion,
        postal_code: &PostalCode,
        streets:     &BTreeSet<StreetName>,
    ) {
        let key = s_key(region, postal_code);
        let val = compress_set_to_cbor(streets);
        db.put(key, val).unwrap();
    }

    #[test]
    fn test_no_street_set_for_postal_code_returns_error() {
        let (data_access, db_arc, _temp_dir) = create_data_access();
        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "12345").unwrap();
        let street = StreetName::new("Main St").unwrap();
        let addr = make_world_address(region, postal_code, street);

        // We store nothing => get_street_set(...) => None
        let result = validate_street_for_postal_code(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::PostalCodeToStreetKeyNotFoundForRegion {
                s_key, region, postal_code
            }) => {
                assert!(
                    s_key.contains("S:"),
                    "Should be an 'S:{region_abbr}:{postalcode}' pattern"
                );
                assert_eq!(postal_code.code(), "12345");
                // region check if needed
            }
            other => panic!("Expected PostalCodeToStreetKeyNotFoundForRegion, got {:?}", other),
        }
    }

    #[test]
    fn test_street_not_in_set_returns_error() {
        let (data_access, db_arc, _temp_dir) = create_data_access();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "21201").unwrap();
        let street_existing = StreetName::new("Howard St").unwrap();
        let street_missing = StreetName::new("NotHere Blvd").unwrap();

        // Insert a set that includes `Howard St` but not `NotHere Blvd`.
        let mut streets = BTreeSet::new();
        streets.insert(street_existing.clone());
        put_s_key_streets(&mut db_guard, &region, &postal_code, &streets);

        let addr = make_world_address(region, postal_code, street_missing.clone());
        let result = validate_street_for_postal_code(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::StreetNotFoundForPostalCodeInRegion {
                street, postal_code, region
            }) => {
                assert_eq!(street.name(), "nothere blvd", "Should match the missing street's normalized name");
                assert_eq!(postal_code.code(), "21201");
            }
            other => panic!("Expected StreetNotFoundForPostalCodeInRegion, got {:?}", other),
        }
    }

    #[test]
    fn test_street_found_returns_ok() {
        let (data_access, db_arc, _temp_dir) = create_data_access();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "21230").unwrap();
        let street = StreetName::new("Fort Ave").unwrap();

        // Insert the street into the set
        let mut streets = BTreeSet::new();
        streets.insert(street.clone());
        put_s_key_streets(&mut db_guard, &region, &postal_code, &streets);

        let addr = make_world_address(region, postal_code, street);
        let result = validate_street_for_postal_code(&addr, &data_access);
        assert!(result.is_ok(), "Street is in the set => Ok(())");
    }

    #[test]
    fn test_corrupted_data_returns_none_and_error() {
        // If the set is invalid CBOR => get_street_set => None => => PostalCodeToStreetKeyNotFoundForRegion
        let (data_access, db_arc, _tmp_dir) = create_data_access();
        let mut db_guard = db_arc.lock().unwrap();

        let region = WorldRegion::try_from_abbreviation("MD").unwrap();
        let postal_code = PostalCode::new(Country::USA, "99999").unwrap();
        let street = StreetName::new("Corrupt St").unwrap();

        let s_k = s_key(&region, &postal_code);
        db_guard.put(s_k.clone(), b"invalid cbor").unwrap();

        let addr = make_world_address(region, postal_code, street);
        let result = validate_street_for_postal_code(&addr, &data_access);
        match result {
            Err(InvalidWorldAddress::PostalCodeToStreetKeyNotFoundForRegion {
                s_key, region, postal_code
            }) => {
                // Because decode fails => None
                assert!(s_key.contains("S:"));
            }
            other => panic!("Expected PostalCodeToStreetKeyNotFoundForRegion, got {:?}", other),
        }
    }
}
