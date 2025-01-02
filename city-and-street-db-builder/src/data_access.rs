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

    pub fn get_postal_code_set(&self, key: &str) -> Option<BTreeSet<PostalCode>> {
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

impl PostalCodesForCityInRegion for DataAccess {

    // Example query: given city name, get associated PostalCode codes
    fn postal_codes_for_city_in_region(&self, region: &WorldRegion, city: &CityName) -> Option<BTreeSet<PostalCode>> {
        let key = c2z_key(region,city);
        if let Some(postal_codes) = self.get_postal_code_set(&key) {
            Some(postal_codes)
        } else {
            None
        }
    }
}

impl StreetNamesForCityInRegion for DataAccess {

    // Similarly for other queries:
    fn street_names_for_city_in_region(&self, region: &WorldRegion, city: &CityName) -> Option<BTreeSet<StreetName>> {
        let key = c2s_key(region,city);
        self.get_street_set(&key)
    }
}

impl CityNamesForPostalCodeInRegion for DataAccess {

    fn cities_for_postal_code(&self, region: &WorldRegion, postal_code: &PostalCode) -> Option<BTreeSet<CityName>> {
        let key = z2c_key(region,postal_code);
        self.get_city_set(&key)
    }
}

impl StreetNamesForPostalCodeInRegion for DataAccess {

    fn street_names_for_postal_code_in_region(
        &self, 
        region: &WorldRegion, 
        postal_code:    &PostalCode

    ) -> Option<BTreeSet<StreetName>> {

        let key = s_key(region,postal_code);
        self.get_street_set(&key)
    }
}

impl StreetExistsInCityInRegion for DataAccess {

    fn street_exists_in_city(
        &self, 
        region: &WorldRegion, 
        city:   &CityName, 
        street: &StreetName

    ) -> bool {

        if let Some(sts) = self.street_names_for_city_in_region(region, city) {
            sts.contains(street)
        } else {
            false
        }
    }
}

impl StreetExistsInPostalCodeInRegion for DataAccess {

    fn street_exists_in_postal_code(&self, region: &WorldRegion, postal_code: &PostalCode, street: &StreetName) -> bool {
        if let Some(sts) = self.street_names_for_postal_code_in_region(region, postal_code) {
            sts.contains(street)
        } else {
            false
        }
    }
}

impl StreetExistsGlobally for DataAccess {

    // street_exists_globally in a region:
    fn street_exists_globally(&self, region: &WorldRegion, street: &StreetName) -> bool {

        // If S2C or S2Z keys exist for this street, it's known:
        let key_cities       = s2c_key(region,street);
        let key_postal_codes = s2z_key(region,street);

        self.get_city_set(&key_cities).is_some() || self.get_postal_code_set(&key_postal_codes).is_some()
    }
}
