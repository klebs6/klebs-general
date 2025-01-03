crate::ix!();

#[derive(Getters)]
#[getset(get="pub")]
pub struct Database {
    db: rocksdb::DB,
}

impl Database {

    /// Initialize DB in its own function
    pub fn open(path: impl AsRef<Path> + Debug) 
        -> Result<Arc<Mutex<Self>>,DatabaseConstructionError> 
    {
        info!("opening rocksdb at path: {:?}", path);

        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Zstd);

        let db = DB::open(&opts, path)?;

        Ok(Arc::new(Mutex::new(Self {
            db
        })))
    }

    /// Check if region already done
    pub fn region_done(&self, region: &WorldRegion) -> Result<bool,rocksdb::Error> {
        Ok(self.db.get(MetaKeyForRegion::from(*region))?.is_some())
    }

    /// Mark region as done
    pub fn mark_region_done(&mut self, region: &WorldRegion) 
        -> Result<(),DatabaseConstructionError> 
    {
        self.db.put(&MetaKeyForRegion::from(*region), b"done")?;
        Ok(())
    }

    pub fn put(&mut self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) -> Result<(),DatabaseConstructionError> {
        self.db.put(key, val)?;
        Ok(())
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Vec<u8>>,DatabaseConstructionError> {
        Ok(self.db.get(key)?)
    }

    /// Write a single region's indexes into DB
    pub fn write_indexes(
        &mut self,
        region:  &WorldRegion,
        indexes: &InMemoryIndexes

    ) -> Result<(),DatabaseConstructionError> {

        info!("writing InMemoryIndexes for region {:?}", region);

        // State->PostalCode->Streets: S:{region}:{postal_code}
        if let Some(state_map) = indexes.postal_code_to_street_map_for_region(region) {
            for (postal_code, streets) in state_map {
                self.write_streets_to_region_and_postal_code(region,postal_code,streets)?;
            }
        }

        // PostalCode->Cities: Z2C:{region_name}:{postal_code}
        for (postal_code, cities) in indexes.postal_code_cities() {
            self.write_cities_to_region_and_postal_code(region,postal_code,cities)?;
        }

        // City->PostalCodes: C2Z:{region_name}:{city}
        for (city, postal_codes) in indexes.city_postal_codes() {
            self.write_postal_codes_to_region_and_city(region,city,postal_codes)?;
        }

        // City->Streets: C2S:{region_name}:{city}
        for (city, streets) in indexes.city_streets() {
            self.write_streets_to_region_and_city(region,city,streets)?;
        }

        // Street->PostalCodes: S2Z:{region_name}:{street}
        for (street, postal_codes) in indexes.street_postal_codes() {
            self.write_postal_codes_to_region_and_street(region,street,postal_codes)?;
        }

        // Street->Cities: S2C:{region_name}:{street}
        for (street, cities) in indexes.street_cities() {
            self.write_cities_to_region_and_street(region,street,cities)?;
        }

        Ok(())
    }

    fn write_streets_to_region_and_postal_code(&mut self, region: &WorldRegion, postal_code: &PostalCode, streets: &BTreeSet<StreetName>) 
        -> Result<(),DatabaseConstructionError> 
    {
        let key = s_key(region,postal_code);
        let val = compress_set_to_cbor(streets);
        self.put(&key, val)?;
        Ok(())
    }

    fn write_cities_to_region_and_postal_code(&mut self, region: &WorldRegion, postal_code: &PostalCode, cities: &BTreeSet<CityName>) 
        -> Result<(),DatabaseConstructionError> 
    {
        let key = z2c_key(region,postal_code);
        let val = compress_set_to_cbor(cities);
        self.put(&key, val)?;
        Ok(())
    }

    fn write_postal_codes_to_region_and_city(&mut self, region: &WorldRegion, city: &CityName, postal_codes: &BTreeSet<PostalCode>) 
        -> Result<(),DatabaseConstructionError> 
    {
        let key = c2z_key(region,city);
        self.put(&key, compress_set_to_cbor(postal_codes))?;
        Ok(())
    }

    fn write_streets_to_region_and_city(&mut self, region: &WorldRegion, city: &CityName, streets: &BTreeSet<StreetName>) -> Result<(), DatabaseConstructionError> {
        let key = c2s_key(region,city);
        self.put(&key, compress_set_to_cbor(streets))?;
        Ok(())
    }

    fn write_postal_codes_to_region_and_street(&mut self, region: &WorldRegion, street: &StreetName, postal_codes: &BTreeSet<PostalCode>) 
        -> Result<(),DatabaseConstructionError> 
    {
        let key = s2z_key(region,street);
        self.put(&key, compress_set_to_cbor(postal_codes))?;
        Ok(())
    }

    fn write_cities_to_region_and_street(&mut self, region: &WorldRegion, street: &StreetName, cities: &BTreeSet<CityName>) 
        -> Result<(),DatabaseConstructionError> 
    {
        let key = s2c_key(region,street);
        self.put(&key, compress_set_to_cbor(cities))?;
        Ok(())
    }
}

/// Tests for Database operations
#[cfg(test)]
mod database_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn open_and_mark_region_done() {

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let db           = Database::open(&temp_dir).unwrap();
        let mut db_guard = db.lock().unwrap();
        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        assert!(!db_guard.region_done(&region).unwrap());

        db_guard.mark_region_done(&region).unwrap();

        assert!(db_guard.region_done(&region).unwrap());
    }
}
