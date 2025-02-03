// ---------------- [ File: src/world_address.rs ]
crate::ix!();

#[derive(Builder,Setters,Getters,Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
#[getset(get="pub",set="pub")]
#[builder(setter(into))]
pub struct WorldAddress {
    region:      WorldRegion,
    postal_code: PostalCode,
    city:        CityName,
    street:      StreetName,
}

impl ValidateWith for WorldAddress {
    type Validator = DataAccess;
    type Error     = InvalidWorldAddress;

    fn validate_with(&self, validator: &Self::Validator) -> Result<(), Self::Error> {
        // 1) Check city <-> postal code
        let z2c_k = z2c_key(&self.region, &self.postal_code);
        match validator.get_city_set(&z2c_k) {
            Some(city_set) => {
                if !city_set.contains(&self.city) {
                    return Err(InvalidWorldAddress::CityNotFoundForPostalCodeInRegion {
                        city: self.city.clone(),
                        postal_code: self.postal_code.clone(),
                        region: self.region,
                    });
                }
            }
            None => {
                return Err(InvalidWorldAddress::PostalCodeToCityKeyNotFoundForRegion {
                    z2c_key: z2c_k,
                    region: self.region,
                    postal_code: self.postal_code.clone(),
                });
            }
        }

        // 2) Check street <-> postal code
        let s_k = s_key(&self.region, &self.postal_code);
        match validator.get_street_set(&s_k) {
            Some(streets) => {
                if !streets.contains(&self.street) {
                    return Err(InvalidWorldAddress::StreetNotFoundForPostalCodeInRegion {
                        street: self.street.clone(),
                        postal_code: self.postal_code.clone(),
                        region: self.region,
                    });
                }
            }
            None => {
                return Err(InvalidWorldAddress::PostalCodeToStreetKeyNotFoundForRegion {
                    s_key: s_k,
                    region: self.region,
                    postal_code: self.postal_code.clone(),
                });
            }
        }

        // 3) Check street <-> city
        let c_k = c_key(&self.region, &self.city);
        match validator.get_street_set(&c_k) {
            Some(streets) => {
                if !streets.contains(&self.street) {
                    return Err(InvalidWorldAddress::StreetNotFoundForCityInRegion {
                        street: self.street.clone(),
                        city:   self.city.clone(),
                        region: self.region,
                    });
                }
            }
            None => {
                return Err(InvalidWorldAddress::CityToStreetsKeyNotFoundForCityInRegion {
                    c_key: c_k,
                    region: self.region,
                    city: self.city.clone(),
                });
            }
        }

        Ok(())
    }
}

// ---------------- [ File: tests/world_address_tests.rs ]
// or you can keep them inline with a `#[cfg(test)] mod world_address_validation_tests;`

#[cfg(test)]
mod world_address_validation_tests {
    use super::*;

    // -----------------------------
    // Confirm the "happy path" works
    // -----------------------------
    #[traced_test]
    fn validate_address_happy_path() {
        // 1) We use the default WorldAddress::mock() => region=VA, city=calverton, street=catlett road, postal=20138-9997.
        //    Ensure your virginia_mock_records() includes (calverton, catlett road, 20138-9997).
        let region_va: WorldRegion = USRegion::UnitedState(UnitedState::Virginia).into();

        // 2) Create DB, store the VA mock data.
        let temp_dir = TempDir::new().expect("temp dir");
        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            let recs = RegionalRecords::mock_for_region(&region_va);
            debug!("regional records for VA: {:#?}", recs);
            recs.write_to_storage(&mut db_guard).unwrap();
        }

        // 3) Validate the default mock address, which should now exist
        let da = DataAccess::with_db(db);
        let address = WorldAddress::mock(); // => region=VA, city=calverton, etc.

        debug!("Testing address: {:#?}", address);
        let result = address.validate_with(&da);
        debug!("Result of validation: {:#?}", result);
        assert!(result.is_ok(), "Expected a valid address with correct region/city/street/postal");
    }

    // -----------------------------
    // Negative test: city mismatch
    // -----------------------------
    #[test]
    fn validate_address_city_not_found_for_postal_code() {
        let region_md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        let temp_dir = TempDir::new().expect("temp dir");
        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            let recs = RegionalRecords::mock_for_region(&region_md);
            recs.write_to_storage(&mut db_guard).unwrap();
        }
        let da = DataAccess::with_db(db.clone());

        // city => "nonexistentcity"
        let addr = WorldAddressBuilder::default()
            .region(region_md)
            .postal_code(PostalCode::new(Country::USA, "21201").unwrap())
            .city(CityName::new("NonExistentCity").unwrap()) // => "nonexistentcity"
            .street(StreetName::new("North Avenue").unwrap()) // => "north avenue"
            .build().unwrap();

        let res = addr.validate_with(&da);
        assert!(res.is_err());
        match res.unwrap_err() {
            InvalidWorldAddress::CityNotFoundForPostalCodeInRegion { city, .. } => {
                assert_eq!(city.name(), "nonexistentcity");
            },
            other => panic!("Expected CityNotFoundForPostalCodeInRegion, got: {:?}", other),
        }
    }

    // -----------------------------
    // Negative test: street mismatch (postal)
    // -----------------------------
    #[test]
    fn validate_address_street_not_found_in_postal_code() {
        let region_md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        let temp_dir = TempDir::new().expect("temp dir");
        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            let recs = RegionalRecords::mock_for_region(&region_md);
            recs.write_to_storage(&mut db_guard).unwrap();
        }

        let da = DataAccess::with_db(db.clone());
        let addr = WorldAddressBuilder::default()
            .region(region_md)
            .postal_code(PostalCode::new(Country::USA, "21201").unwrap())
            .city(CityName::new("Baltimore").unwrap()) // => "baltimore"
            .street(StreetName::new("Imaginary Lane").unwrap()) // => "imaginary lane"
            .build().unwrap();

        let res = addr.validate_with(&da);
        assert!(res.is_err());
        match res.unwrap_err() {
            InvalidWorldAddress::StreetNotFoundForPostalCodeInRegion { street, .. } => {
                assert_eq!(street.name(), "imaginary lane");
            },
            other => panic!("Expected StreetNotFoundForPostalCodeInRegion, got: {:?}", other),
        }
    }

    // -----------------------------
    // Negative test: *forced* partial DB to produce
    // "StreetNotFoundForCityInRegion"
    // -----------------------------
    #[test]
    fn validate_address_street_not_found_in_city() {
        // We'll forcibly insert partial data:
        //  * postal->city => city= "rockville"
        //  * postal->street => "main street"
        //  * city->postal => 20850
        //  * city->street => a set that has "fake street" but *not* "main street".
        // This ensures the first checks pass, but the final city->street membership fails.

        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(&temp_dir).unwrap();

        {
            let mut db_guard = db.lock().unwrap();
            let region_md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

            let city = CityName::new("Rockville").unwrap();     // => "rockville"
            let postal = PostalCode::new(Country::USA, "20850").unwrap();

            // Step #1: S:US:20850 => {"main street"}
            let s_k = s_key(&region_md, &postal);
            let mut streets_for_postal = std::collections::BTreeSet::new();
            streets_for_postal.insert(StreetName::new("main street").unwrap());
            db_guard
                .put(&s_k, crate::compress_set_to_cbor(&streets_for_postal))
                .unwrap();

            // Step #2: Z2C:US:20850 => {"rockville"}
            let z2c_k = z2c_key(&region_md, &postal);
            let mut cities_for_postal = std::collections::BTreeSet::new();
            cities_for_postal.insert(city.clone());
            db_guard
                .put(&z2c_k, crate::compress_set_to_cbor(&cities_for_postal))
                .unwrap();

            // Step #3: C2Z:US:rockville => {20850}
            let c2z_k = c2z_key(&region_md, &city);
            let mut postals_for_city = std::collections::BTreeSet::new();
            postals_for_city.insert(postal.clone());
            db_guard
                .put(&c2z_k, crate::compress_set_to_cbor(&postals_for_city))
                .unwrap();

            // Step #4: C2S:US:rockville => some set that does NOT contain "main street".
            // If we stored an empty set, your DataAccess code returns `None` => "CityToStreetsKeyNotFoundForCityInRegion".
            // Instead, we store a different street => membership check fails => "StreetNotFoundForCityInRegion".
            let c2s_k = c2s_key(&region_md, &city);
            let mut city_streets = std::collections::BTreeSet::new();
            city_streets.insert(StreetName::new("fake street").unwrap());
            db_guard
                .put(&c2s_k, crate::compress_set_to_cbor(&city_streets))
                .unwrap();

            // Once we exit this block, the `db_guard` is dropped, avoiding a deadlock.
            }

        // Now we can call `validate_with(...)` freely.
        let da = DataAccess::with_db(db.clone());

        let address = WorldAddressBuilder::default()
            .region::<WorldRegion>(USRegion::UnitedState(UnitedState::Maryland).into())
            .postal_code(PostalCode::new(Country::USA, "20850").unwrap())
            .city(CityName::new("Rockville").unwrap())   // => "rockville"
            .street(StreetName::new("Main Street").unwrap()) // => "main street"
            .build()
            .unwrap();

        let result = address.validate_with(&da);
        assert!(
            result.is_err(),
            "Expected street not found in city at final check"
        );

        match result.unwrap_err() {
            InvalidWorldAddress::StreetNotFoundForCityInRegion { street, city, .. } => {
                assert_eq!(street.name(), "main street");
                assert_eq!(city.name(), "rockville");
            }
            other => panic!("Expected StreetNotFoundForCityInRegion, got: {:?}", other),
        }
    }

    // -----------------------------
    // Negative test: region mismatch
    // -----------------------------
    #[test]
    fn validate_address_region_mismatch() {
        // We'll store only MD data, but create a VA address => "PostalCodeToCityKeyNotFoundForRegion"
        let region_md: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            let recs = RegionalRecords::mock_for_region(&region_md);
            recs.write_to_storage(&mut db_guard).unwrap();
        }

        let da = DataAccess::with_db(db.clone());

        // region=VA => "calverton," "catlett road," "20138-9997" not in DB
        let addr = WorldAddress::mock();
        let res = addr.validate_with(&da);
        assert!(res.is_err());
        match res.unwrap_err() {
            InvalidWorldAddress::PostalCodeToCityKeyNotFoundForRegion { postal_code, .. } => {
                assert_eq!(postal_code.code(), "20138-9997");
            }
            other => panic!("Expected PostalCodeToCityKeyNotFoundForRegion, got: {:?}", other),
        }
    }

    // -----------------------------
    // Quick check: missing region in builder
    // -----------------------------
    #[test]
    fn validate_address_builder_missing_region() {
        let attempt = WorldAddressBuilder::default()
            .postal_code(PostalCode::new(Country::USA,"21201").unwrap())
            .city(CityName::new("Baltimore").unwrap())
            .street(StreetName::new("North Avenue").unwrap())
            .build(); 
        assert!(attempt.is_err(), "Region is mandatory");
    }
}
