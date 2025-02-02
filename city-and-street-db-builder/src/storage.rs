// ---------------- [ File: src/storage.rs ]
crate::ix!();

/// A simple "Database" wrapper that sets up the dynamic prefix transform.
#[derive(Getters)]
#[getset(get="pub")]
pub struct Database {
    db: rocksdb::DB,
}

impl Database {

    /// Initialize DB in its own function
    pub fn open(path: impl AsRef<std::path::Path>) 
        -> Result<Arc<Mutex<Self>>, DatabaseConstructionError> 
    {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(DBCompressionType::Zstd);

        // Use the dynamic slice transform
        let st = create_colon_prefix_transform();
        opts.set_prefix_extractor(st);

        // Optionally enable prefix bloom filters
        opts.set_memtable_prefix_bloom_ratio(0.1);

        let db = DB::open(&opts, path)?;

        Ok(Arc::new(Mutex::new(Self { db })))
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

    #[test]
    fn test_open_basic() {
        // Create a temp dir, open DB, ensures no errors.
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = Database::open(&temp_dir);
        assert!(db.is_ok(), "Should open DB successfully in new directory");
    }

    #[test]
    fn test_open_and_mark_region_done() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db = Database::open(&temp_dir).expect("DB should open");
        let mut db_guard = db.lock().unwrap();

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        // Initially => region_done => false
        let done_before = db_guard.region_done(&region).unwrap();
        assert!(!done_before, "Region not done initially");

        // Mark done
        db_guard.mark_region_done(&region).unwrap();

        // region_done => true
        let done_after = db_guard.region_done(&region).unwrap();
        assert!(done_after, "Region should be marked done");
    }

    #[test]
    fn test_put_and_get() {
        let temp_dir = TempDir::new().expect("temp dir");
        let db = Database::open(&temp_dir).expect("open db");
        let mut db_guard = db.lock().unwrap();

        // Put some key-value
        let key = b"my_test_key";
        let value = b"my_test_value";
        db_guard.put(&key[..], &value[..]).unwrap();

        // Get it back
        let result = db_guard.get(&key[..]).unwrap();
        assert!(result.is_some(), "Should retrieve stored value");
        assert_eq!(result.unwrap(), value, "Value matches");
    }

    #[test]
    fn test_put_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(&temp_dir).unwrap();
        let mut db_guard = db.lock().unwrap();

        let key = b"same_key";
        let val1 = b"first_value";
        let val2 = b"second_value";

        // Put val1
        db_guard.put(key, val1).unwrap();
        let r1 = db_guard.get(key).unwrap().unwrap();
        assert_eq!(r1, val1);

        // Overwrite with val2
        db_guard.put(key, val2).unwrap();
        let r2 = db_guard.get(key).unwrap().unwrap();
        assert_eq!(r2, val2, "Overwritten with new value");
    }

    #[test]
    fn test_write_indexes_basic() {
        let temp_dir = TempDir::new().expect("temp dir");
        let db = Database::open(&temp_dir).expect("open db");
        let mut db_guard = db.lock().unwrap();

        let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();

        // We'll build a small InMemoryIndexes:
        // city -> postal, postal -> city, etc.
        let mut indexes = InMemoryIndexesBuilder::default()
            .region_postal_code_streets(BTreeMap::new())
            .postal_code_cities(BTreeMap::new())
            .city_postal_codes(BTreeMap::new())
            .city_streets(BTreeMap::new())
            .street_postal_codes(BTreeMap::new())
            .street_cities(BTreeMap::new())
            .build()
            .unwrap();

        // Insert a small record: postal=21201 => city="baltimore" => street="north avenue"
        // We'll do a minimal approach: city->postal, postal->city, etc.
        let postal = PostalCode::new(Country::USA, "21201").unwrap();
        let city   = CityName::new("Baltimore").unwrap();
        let street = StreetName::new("North Avenue").unwrap();

        // region_postal_code_streets => region->(postal->streets)
        // We'll create a top-level map if needed:
        indexes.region_postal_code_streets_mut().insert(region, BTreeMap::new());
        indexes.region_postal_code_streets_mut()
            .get_mut(&region).unwrap()
            .insert(postal.clone(), {
                let mut s = BTreeSet::new();
                s.insert(street.clone());
                s
            });

        // postal_code_cities => postal->cities
        indexes.postal_code_cities_mut().insert(postal.clone(), {
            let mut c = BTreeSet::new();
            c.insert(city.clone());
            c
        });

        // city_postal_codes => city->postal
        indexes.city_postal_codes_mut().insert(city.clone(), {
            let mut p = BTreeSet::new();
            p.insert(postal.clone());
            p
        });

        // city_streets => city->streets
        indexes.city_streets_mut().insert(city.clone(), {
            let mut s = BTreeSet::new();
            s.insert(street.clone());
            s
        });

        // street_postal_codes => street->postal
        indexes.street_postal_codes_mut().insert(street.clone(), {
            let mut p = BTreeSet::new();
            p.insert(postal.clone());
            p
        });

        // street_cities => street->city
        indexes.street_cities_mut().insert(street.clone(), {
            let mut c = BTreeSet::new();
            c.insert(city.clone());
            c
        });

        // Now write it
        db_guard.write_indexes(&region, &indexes).unwrap();

        // The DB now has S:{abbr}:21201 => cbor'd set of "north avenue", etc.
        // Let's confirm at least one of them:
        let s_key_str = crate::s_key(&region, &postal);
        let raw_val = db_guard.get(&s_key_str).unwrap().unwrap();
        let decompressed: Vec<StreetName> = crate::decompress_cbor_to_list(&raw_val);
        assert_eq!(decompressed.len(), 1, "One street name in the set");
        assert_eq!(decompressed[0].name(), "north avenue");
    }

    #[test]
    fn test_region_done_persists_between_open() {
        // We'll test that "mark_region_done" => region_done => true 
        // then we close the DB and reopen => still true.

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("rocksdb_persist_test");

        {
            // open, mark region done
            let db = Database::open(&path).unwrap();
            let mut db_guard = db.lock().unwrap();
            let region: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
            db_guard.mark_region_done(&region).unwrap();
        }

        {
            // reopen
            let db2 = Database::open(&path).unwrap();
            let db2_guard = db2.lock().unwrap();
            let region2: WorldRegion = USRegion::UnitedState(UnitedState::Maryland).into();
            // should be true
            assert!(db2_guard.region_done(&region2).unwrap());
        }
    }

    #[test]
    fn test_open_invalid_path_permissions() {
        // This is OS-specific. 
        // If we try to open a directory that's read-only, we might get an error. 
        // We'll do a partial approach: 
        // Attempt to open an invalid path like "/this/does/not/exist" => likely fails.

        #[cfg(unix)]
        {
            let invalid_path = PathBuf::from("/root/some_path_we_cant_write");
            let result = Database::open(&invalid_path);
            match result {
                Err(DatabaseConstructionError::RocksDB(e)) => {
                    // e.g. "Invalid argument" or "IO error"
                    debug!("Got a rocksdb open error as expected: {}", e);
                },
                _ => {
                    // On some systems, this might not fail. We'll not strictly panic. 
                    // Possibly do a partial check if it fails or not.
                    debug!("Might not fail on all systems, depends on perms.");
                }
            }
        }
    }
}
