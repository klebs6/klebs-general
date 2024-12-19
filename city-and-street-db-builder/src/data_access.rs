crate::ix!();

/// DataAccess struct for queries
pub struct DataAccess {
    db: Arc<Mutex<Database>>,
}

impl DataAccess {

    pub fn with_db(db: Arc<Mutex<Database>>) -> Self {
        info!("creating DataAccess object");
        DataAccess { db }
    }

    pub fn get_city_set(&self, key: &str) -> Option<BTreeSet<CityName>> {
        self.get_cbor_set_typed::<CityName>(key)
    }

    pub fn get_street_set(&self, key: &str) -> Option<BTreeSet<StreetName>> {
        self.get_cbor_set_typed::<StreetName>(key)
    }

    pub fn get_zip_set(&self, key: &str) -> Option<BTreeSet<PostalCode>> {
        self.get_cbor_set_typed::<PostalCode>(key)
    }

    fn get_cbor_set_typed<T>(&self, key: &str) -> Option<BTreeSet<T>>
    where
        T: Serialize + DeserializeOwned + Ord,
    {
        match self.db.lock() {
            Ok(db) => {
                let val = db.get(key).ok()??;
                let list: Vec<T> = decompress_cbor_to_list(&val);
                if list.is_empty() {
                    None
                } else {
                    Some(list.into_iter().collect())
                }
            }
            Err(_) => {
                warn!("Could not get DB lock for key: {}", key);
                None
            },
        }
    }
}

impl ZipCodesForCityInRegion for DataAccess {

    // Example query: given city name, get associated ZIP codes
    fn zip_codes_for_city_in_region(&self, region: &USRegion, city: &CityName) -> Option<BTreeSet<PostalCode>> {
        let key = c2z_key(region,city);
        if let Some(zips) = self.get_zip_set(&key) {
            Some(zips)
        } else {
            None
        }
    }
}

impl StreetNamesForCityInRegion for DataAccess {

    // Similarly for other queries:
    fn street_names_for_city_in_region(&self, region: &USRegion, city: &CityName) -> Option<BTreeSet<StreetName>> {
        let key = c2s_key(region,city);
        self.get_street_set(&key)
    }
}

impl CityNamesForZipCodeInRegion for DataAccess {

    fn cities_for_zip(&self, region: &USRegion, zip: &PostalCode) -> Option<BTreeSet<CityName>> {
        let key = z2c_key(region,zip);
        self.get_city_set(&key)
    }
}

impl StreetNamesForZipCodeInRegion for DataAccess {

    fn street_names_for_zip_code_in_region(
        &self, 
        region: &USRegion, 
        zip:    &PostalCode

    ) -> Option<BTreeSet<StreetName>> {

        let key = s_key(region,zip);
        self.get_street_set(&key)
    }
}

impl StreetExistsInCityInRegion for DataAccess {

    fn street_exists_in_city(
        &self, 
        region: &USRegion, 
        city:        &CityName, 
        street:      &StreetName

    ) -> bool {

        if let Some(sts) = self.street_names_for_city_in_region(region, city) {
            sts.contains(street)
        } else {
            false
        }
    }
}

impl StreetExistsInZipCodeInRegion for DataAccess {

    fn street_exists_in_zip(&self, region: &USRegion, zip: &PostalCode, street: &StreetName) -> bool {
        if let Some(sts) = self.street_names_for_zip_code_in_region(region, zip) {
            sts.contains(street)
        } else {
            false
        }
    }
}

impl StreetExistsGlobally for DataAccess {

    // street_exists_globally in a region:
    fn street_exists_globally(&self, region: &USRegion, street: &StreetName) -> bool {
        // If S2C or S2Z keys exist for this street, it's known:
        let key_cities = s2c_key(region,street);
        let key_zips   = s2z_key(region,street);

        self.get_city_set(&key_cities).is_some() || self.get_zip_set(&key_zips).is_some()
    }
}
