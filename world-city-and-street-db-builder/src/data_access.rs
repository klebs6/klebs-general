// ---------------- [ File: src/data_access.rs ]
crate::ix!();

/// DataAccess struct for queries
#[derive(Clone)]
pub struct DataAccess {
    db: Arc<Mutex<Database>>,
}

impl DataAccess {

    /// Creates a new DataAccess that wraps the given Database (thread-safe).
    pub fn with_db(db: Arc<Mutex<Database>>) -> Self {
        info!("creating DataAccess object");
        DataAccess { db }
    }

    /// Returns a set of CityName objects for the given string key, if present.
    pub fn get_city_set(&self, key: &str) -> Option<BTreeSet<CityName>> {
        self.get_cbor_set_typed::<CityName>(key)
    }

    /// Returns a set of StreetName objects for the given string key, if present.
    pub fn get_street_set(&self, key: &str) -> Option<BTreeSet<StreetName>> {
        self.get_cbor_set_typed::<StreetName>(key)
    }

    /// Returns a set of PostalCode objects for the given string key, if present.
    pub fn get_postal_code_set(&self, key: &str) -> Option<BTreeSet<PostalCode>> {
        self.get_cbor_set_typed::<PostalCode>(key)
    }

    /// INTERNAL HELPER: fetch a CBOR-encoded BTreeSet<T> from DB under `key`.
    /// If key not present or empty, returns None. If DB lock is poisoned,
    /// logs warning and returns None.
    fn get_cbor_set_typed<T>(&self, key: &str) -> Option<BTreeSet<T>>
    where
        T: Serialize + DeserializeOwned + Ord,
    {
        match self.db.lock() {
            Ok(db_guard) => {
                let val = match db_guard.get(key) {
                    Ok(opt) => opt,
                    Err(e) => {
                        warn!("DB get error for key {}: {}", key, e);
                        return None;
                    }
                };
                let bytes = val?;
                let list: Vec<T> = decompress_cbor_to_list(&bytes);
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

//-----------------------------------------------
impl GatherAllZipsInRegion for DataAccess {

    // -----------------------------------------------------------
    // (B) The integrated function to gather ALL zips in a region.
    // -----------------------------------------------------------
    //
    // This iterates over the DB keys that start with `Z2C:<region_abbr>:` 
    // and extracts the postal code substring from each key. 
    // If the code is valid, we add it to a Vec.
    //
    fn gather_all_zips_in_region(&self, region: &WorldRegion) -> Vec<PostalCode> {
        let prefix = format!("Z2C:{}:", region.abbreviation());
        match self.db.lock() {
            Ok(db_guard) => {
                let iter = db_guard.db().prefix_iterator(prefix.as_bytes());
                let mut out = Vec::new();
                for kv in iter {
                    if let Ok((key_bytes, _val_bytes)) = kv {
                        let key_str = String::from_utf8_lossy(&key_bytes);
                        // e.g. "Z2C:US:21201"
                        let parts: Vec<&str> = key_str.splitn(3, ':').collect();
                        if parts.len() < 3 {
                            continue;
                        }
                        let zip_str = parts[2];
                        if let Ok(pc) = PostalCode::new(Country::USA, zip_str) {
                            out.push(pc);
                        }
                    }
                }
                out
            },
            Err(_) => {
                warn!("Could not lock DB in gather_all_zips_in_region");
                Vec::new()
            }
        }
    }
}

//-----------------------------------------------
impl LoadHouseNumberRanges for DataAccess {

    // ----------------------------------------------------------------------
    // (C) Example method to load house-number ranges from DB or a stub
    // ----------------------------------------------------------------------
    //
    // This is purely illustrative. Adjust the signature or error handling 
    // as needed in your codebase.
    //
    fn load_house_number_ranges(
        &self, 
        region: &WorldRegion, 
        street_obj: &StreetName
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> 
    {
        match self.db.lock() {
            Ok(db_guard) => {
                // Suppose we have a function `load_house_number_ranges_impl(...)`
                // that you define for reading from the DB or a stub. 
                // We'll inline a minimal version here for demonstration:
                let results = load_house_number_ranges_impl(&db_guard, region, street_obj)?;
                Ok(results)
            }
            Err(_) => {
                warn!("Could not get DB lock for house number ranges");
                // Return Ok(None) or an error, your choice. We'll do Ok(None).
                Ok(None)
            },
        }
    }
}

// This is a dummy "impl" that tries to read from DB with prefix "S2R:<region_abbr>:<street>"
fn load_house_number_ranges_impl(
    db: &Database,
    region: &WorldRegion,
    street: &StreetName,
) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
    let key = format!("S2R:{}:{}", region.abbreviation(), street.name());
    let val_opt = db.get(key.as_bytes())?; // might return Option<Vec<u8>>

    let raw_bytes = match val_opt {
        Some(b) => b,
        None => return Ok(None),
    };
    let decoded: Vec<HouseNumberRange> = crate::compressed_list::decompress_cbor_to_list(&raw_bytes);
    if decoded.is_empty() {
        Ok(None)
    } else {
        Ok(Some(decoded))
    }
}

#[cfg(test)]
mod data_access_tests {

    use super::*;

    /// Creates a fresh Database + DataAccess. 
    /// The question mark operator (`?`) will automatically convert errors 
    /// to DataAccessError, thanks to error_tree! definitions.
    fn create_db_and_da() -> Result<(Arc<Mutex<Database>>, DataAccess), DataAccessError> {
        let tmp = TempDir::new()?;                // => DataAccessError::Io if fails
        let db = Database::open(tmp.path())?;     // => DataAccessError::DatabaseConstructionError if fails
        let da = DataAccess::with_db(db.clone());
        Ok((db, da))
    }

    /// Helper: writes a BTreeSet<T> as CBOR into the DB at `key`.
    fn put_set_into_db<T: serde::Serialize + serde::de::DeserializeOwned + Ord + Clone>(
        db: &mut Database,
        key: &str,
        items: &BTreeSet<T>
    ) -> Result<(), DataAccessError> {
        let cbor_data = compress_set_to_cbor(items);
        db.put(key, cbor_data)?; // => DataAccessError::DatabaseConstructionError
        Ok(())
    }

    // Some sample “constructor” functions:
    fn region_md() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Maryland).into()
    }
    fn region_va() -> WorldRegion {
        USRegion::UnitedState(UnitedState::Virginia).into()
    }

    fn city_baltimore() -> CityName {
        CityName::new("Baltimore").unwrap()
    }
    fn city_calverton() -> CityName {
        CityName::new("Calverton").unwrap()
    }

    fn street_north_avenue() -> StreetName {
        StreetName::new("North Avenue").unwrap()
    }
    fn street_catlett_road() -> StreetName {
        StreetName::new("Catlett Road").unwrap()
    }

    fn postal_21201() -> PostalCode {
        PostalCode::new(Country::USA, "21201").unwrap()
    }
    fn postal_20138_9997() -> PostalCode {
        PostalCode::new(Country::USA, "20138-9997").unwrap()
    }

    // --------------------------------------------
    // Basic .get_xxx_set usage
    // --------------------------------------------

    #[test]
    fn test_get_city_set_no_key() -> Result<(), DataAccessError> {
        let (_db, da) = create_db_and_da()?;
        let result = da.get_city_set("Z2C:US:99999");
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn test_get_city_set_valid() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut db_guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = "Z2C:US:21201";
            let mut city_set = BTreeSet::new();
            city_set.insert(city_baltimore());
            put_set_into_db(&mut db_guard, key, &city_set)?;
        }
        let found = da.get_city_set("Z2C:US:21201");
        assert!(found.is_some());
        let set = found.unwrap();
        assert_eq!(set.len(), 1);
        assert!(set.contains(&city_baltimore()));
        Ok(())
    }

    #[test]
    fn test_get_city_set_empty() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut db_guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = "Z2C:US:EMPTY";
            let empty: BTreeSet<CityName> = BTreeSet::new();
            put_set_into_db(&mut db_guard, key, &empty)?;
        }
        let found = da.get_city_set("Z2C:US:EMPTY");
        assert!(found.is_none(), "Empty => None");
        Ok(())
    }

    #[test]
    fn test_get_street_set_single() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut db_guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = "C2S:US:baltimore";
            let mut st_set = BTreeSet::new();
            st_set.insert(street_north_avenue());
            put_set_into_db(&mut db_guard, key, &st_set)?;
        }
        let found = da.get_street_set("C2S:US:baltimore");
        assert!(found.is_some());
        let st = found.unwrap();
        assert!(st.contains(&street_north_avenue()));
        Ok(())
    }

    #[test]
    fn test_get_postal_code_set_multiple() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut db_guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = "C2Z:US:baltimore";
            let mut zips = BTreeSet::new();
            zips.insert(postal_21201());
            zips.insert(PostalCode::new(Country::USA, "21202").unwrap());
            put_set_into_db(&mut db_guard, key, &zips)?;
        }
        let found = da.get_postal_code_set("C2Z:US:baltimore");
        assert!(found.is_some());
        let codes = found.unwrap();
        assert_eq!(codes.len(), 2);
        Ok(())
    }

    // --------------------------------------------
    // Testing the trait-based convenience methods
    // --------------------------------------------

    #[test]
    fn test_postal_codes_for_city_in_region() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = c2z_key(&region_md(), &city_baltimore());
            let mut set = BTreeSet::new();
            set.insert(postal_21201());
            put_set_into_db(&mut guard, &key, &set)?;
        }
        let found = da.postal_codes_for_city_in_region(&region_md(), &city_baltimore());
        assert!(found.is_some());
        let s = found.unwrap();
        assert!(s.contains(&postal_21201()));
        Ok(())
    }

    #[test]
    fn test_street_names_for_city_in_region() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = c2s_key(&region_md(), &city_baltimore());
            let mut st = BTreeSet::new();
            st.insert(street_north_avenue());
            put_set_into_db(&mut guard, &key, &st)?;
        }
        let found = da.street_names_for_city_in_region(&region_md(), &city_baltimore());
        assert!(found.is_some());
        assert!(found.unwrap().contains(&street_north_avenue()));
        Ok(())
    }

    #[test]
    fn test_cities_for_postal_code() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = z2c_key(&region_md(), &postal_21201());
            let mut cset = BTreeSet::new();
            cset.insert(city_baltimore());
            put_set_into_db(&mut guard, &key, &cset)?;
        }
        let found = da.cities_for_postal_code(&region_md(), &postal_21201());
        assert!(found.is_some());
        assert!(found.unwrap().contains(&city_baltimore()));
        Ok(())
    }

    #[test]
    fn test_street_names_for_postal_code_in_region() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = s_key(&region_md(), &postal_21201());
            let mut st = BTreeSet::new();
            st.insert(street_north_avenue());
            put_set_into_db(&mut guard, &key, &st)?;
        }
        let found = da.street_names_for_postal_code_in_region(&region_md(), &postal_21201());
        assert!(found.is_some());
        assert!(found.unwrap().contains(&street_north_avenue()));
        Ok(())
    }

    // --------------------------------------------
    // Checking existence queries
    // --------------------------------------------

    #[test]
    fn test_street_exists_in_city_true() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = c2s_key(&region_md(), &city_baltimore());
            let mut st = BTreeSet::new();
            st.insert(street_north_avenue());
            put_set_into_db(&mut guard, &key, &st)?;
        }
        let exists = da.street_exists_in_city(&region_md(), &city_baltimore(), &street_north_avenue());
        assert!(exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_in_city_false() -> Result<(), DataAccessError> {
        let (_db, da) = create_db_and_da()?;
        let exists = da.street_exists_in_city(&region_md(), &city_baltimore(), &street_north_avenue());
        assert!(!exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_in_postal_code_true() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let key = s_key(&region_md(), &postal_21201());
            let mut st = BTreeSet::new();
            st.insert(street_north_avenue());
            put_set_into_db(&mut guard, &key, &st)?;
        }
        let exists = da.street_exists_in_postal_code(&region_md(), &postal_21201(), &street_north_avenue());
        assert!(exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_in_postal_code_false() -> Result<(), DataAccessError> {
        let (_db, da) = create_db_and_da()?;
        let exists = da.street_exists_in_postal_code(&region_md(), &postal_21201(), &street_north_avenue());
        assert!(!exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_globally_true() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        {
            let mut guard = db.lock().map_err(|_| DataAccessError::LockPoisoned)?;
            let region = region_va();
            let street = street_catlett_road();

            let s2c_k = s2c_key(&region, &street);
            let mut cities = BTreeSet::new();
            cities.insert(city_calverton());
            put_set_into_db(&mut guard, &s2c_k, &cities)?;

            let s2z_k = s2z_key(&region, &street);
            let mut zips = BTreeSet::new();
            zips.insert(postal_20138_9997());
            put_set_into_db(&mut guard, &s2z_k, &zips)?;
        }
        let region = region_va();
        let street = street_catlett_road();
        let exists = da.street_exists_globally(&region, &street);
        assert!(exists);
        Ok(())
    }

    #[test]
    fn test_street_exists_globally_false() -> Result<(), DataAccessError> {
        let (_db, da) = create_db_and_da()?;
        let region = region_va();
        let street = street_catlett_road();
        let exists = da.street_exists_globally(&region, &street);
        assert!(!exists);
        Ok(())
    }

    // --------------------------------------------
    // Lock poisoning scenario
    // --------------------------------------------

    #[test]
    fn test_lock_poisoning_logged_as_warning() -> Result<(), DataAccessError> {
        let (db, da) = create_db_and_da()?;
        // Force a lock poison:
        let _ = std::panic::catch_unwind(|| {
            let guard = db.lock().unwrap();
            let _ = &guard; // just hold it
            panic!("Intentionally poisoning the lock");
        });
        // Now the lock is poisoned. 
        // The data_access calls `db.lock()`, sees an error => DataAccessError::LockPoisoned => logs -> returns None
        let val = da.get_city_set("some_key");
        assert!(val.is_none());
        Ok(())
    }
}
