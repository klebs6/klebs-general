crate::ix!();

#[derive(Builder,Setters,Getters,Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
#[getset(get="pub",set="pub")]
#[builder(setter(into))]
pub struct UsaAddress {
    region: USRegion,
    zip:    PostalCode,
    city:   CityName,
    street: StreetName,
}

impl ValidateWith for UsaAddress {
    type Validator = DataAccess;
    type Error     = InvalidUsaAddress;

    fn validate_with(
        &self, 
        validator: &Self::Validator,
    ) -> Result<(),Self::Error> {

        let z2c_key = z2c_key(&self.region,&self.zip);
        let city_set = validator.get_city_set(&z2c_key);

        match city_set {
            Some(cities) => {
                if !cities.contains(&self.city) {
                    info!("cities: {:#?}", cities);
                    return Err(InvalidUsaAddress::CityNotFoundForZipCodeInRegion {
                        city:   self.city.clone(),
                        zip:    self.zip.clone(),
                        region: self.region,
                    });
                }
            },
            None => {
                return Err(InvalidUsaAddress::ZipToCityKeyNotFoundForRegion {
                    z2c_key,
                    region: self.region,
                    zip:    self.zip.clone(),
                });
            }
        }

        let s_key = s_key(&self.region,&self.zip);
        let street_set_for_zip = validator.get_street_set(&s_key);

        match street_set_for_zip {
            Some(streets) => {
                if !streets.contains(&self.street) {
                    info!("streets: {:#?}", streets);
                    return Err(InvalidUsaAddress::StreetNotFoundForZipCodeInRegion {
                        street: self.street.clone(),
                        zip:    self.zip.clone(),
                        region: self.region,
                    });
                }
            },
            None => {
                return Err(InvalidUsaAddress::ZipToStreetKeyNotFoundForRegion {
                    s_key,
                    region: self.region,
                    zip:    self.zip.clone(),
                });
            }
        }

        let c_key = c_key(&self.region,&self.city);
        let street_set_for_city = validator.get_street_set(&c_key);

        match street_set_for_city {
            Some(streets) => {
                if !streets.contains(&self.street) {
                    info!("streets: {:#?}", streets);
                    return Err(
                        InvalidUsaAddress::StreetNotFoundForCityInRegion {
                            street: self.street.clone(),
                            city:   self.city.clone(),
                            region: self.region,
                        }
                    );
                }
            },
            None => {
                return Err(InvalidUsaAddress::CityToStreetsKeyNotFoundForCityInRegion {
                    c_key,
                    region: self.region,
                    city:   self.city.clone(),
                });
            }
        }

        Ok(())
    }
}

/// Tests for ValidateWith on UsaAddress
#[cfg(test)]
mod usa_address_validation_tests {
    use super::*;

    #[traced_test]
    fn validate_address_happy_path() {

        let region = USRegion::UnitedState(UnitedState::Maryland);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Use mock data + DataAccess
        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            let rr = RegionalRecords::mock_for_region(&region);
            debug!("rr: {:#?}", rr);
            rr.write_to_storage(&mut *db_guard).unwrap();
            db_guard.dump_entire_database_contents();
        }
        let da = DataAccess::with_db(db.clone());
        let address = UsaAddress::mock();
        debug!("address: {:#?}", address);
        let res = address.validate_with(&da);
        debug!("res: {:#?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn validate_address_city_not_found_for_zip() {

        let region = USRegion::UnitedState(UnitedState::Maryland);

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let db = Database::open(&temp_dir).unwrap();
        {
            let mut db_guard = db.lock().unwrap();
            let rr = RegionalRecords::mock_for_region(&region);
            rr.write_to_storage(&mut *db_guard).unwrap();
        }
        let da = DataAccess::with_db(db.clone());

        // Create an address with a non-existent city for given zip:
        let address = UsaAddressBuilder::default()
            .region(USRegion::UnitedState(UnitedState::Maryland))
            .zip(PostalCode::new(Country::USA,"21201").unwrap())
            .city(CityName::new("NonExistentCity").unwrap())
            .street(StreetName::new("North Avenue").unwrap())
            .build()
            .unwrap();

        let res = address.validate_with(&da);
        assert!(res.is_err());
        match res.err().unwrap() {
            InvalidUsaAddress::CityNotFoundForZipCodeInRegion { city, .. } => {
                assert_eq!(city.name(), "NonExistentCity");
            },
            _ => panic!("Expected CityNotFoundForZipCodeInRegion"),
        }
    }
}
