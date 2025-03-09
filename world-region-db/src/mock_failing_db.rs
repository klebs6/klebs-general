// ---------------- [ File: src/mock_failing_db.rs ]
crate::ix!();

// If put fails => returns DatabaseConstructionError::RocksDB
pub struct FailingDbStub;

impl StoreHouseNumberRanges for FailingDbStub {
    fn store_house_number_ranges(
        &mut self,
        region: &WorldRegion,
        street: &StreetName,
        _ranges: &[HouseNumberRange],
    ) -> Result<(), DatabaseConstructionError> 
    {
        let key = house_number_ranges_key(region, street);
        // We'll skip actual cbor logic for brevity
        let data = vec![1,2,3];
        self.put(key.as_bytes(), data)?;
        Ok(())
    }
}

impl DatabasePut for FailingDbStub {
    fn put(
        &mut self, 
        _key: impl AsRef<[u8]>, 
        _val: impl AsRef<[u8]>
    ) -> Result<(), DatabaseConstructionError> {
        Err(DatabaseConstructionError::SimulatedStoreFailure)
    }
}
// Combine with trait
impl OpenDatabaseAtPath for FailingDbStub {
    fn open(_p: impl AsRef<std::path::Path>) 
        -> Result<Arc<Mutex<Self>>, WorldCityAndStreetDbBuilderError> 
    {
        unimplemented!()
    }
}

impl WriteStreetsToRegionAndPostalCode for FailingDbStub {
    fn write_streets_to_region_and_postal_code(
        &mut self, 
        region: &WorldRegion, 
        postal_code: &PostalCode, 
        streets: &BTreeSet<StreetName>
    ) -> Result<(),DatabaseConstructionError> {
        let key = s_key(region, postal_code);
        let val = compress_set_to_cbor(streets);
        self.put(&key, val)?; // forcibly fails
        Ok(())
    }
}

impl WriteStreetsToRegionAndCity for FailingDbStub {
    fn write_streets_to_region_and_city(
        &mut self, 
        region: &WorldRegion, 
        city: &CityName, 
        streets: &BTreeSet<StreetName>
    ) -> Result<(), DatabaseConstructionError> {
        let key = c2s_key(region, city);
        let val = compress_set_to_cbor(streets);
        self.put(key, val)?; // forcibly fails
        Ok(())
    }
}

impl WritePostalCodesToRegionAndStreet for FailingDbStub {
    fn write_postal_codes_to_region_and_street(
        &mut self, 
        region: &WorldRegion, 
        street: &StreetName, 
        postal_codes: &BTreeSet<PostalCode>
    ) -> Result<(),DatabaseConstructionError> {
        let key = s2z_key(region, street);
        let val = compress_set_to_cbor(postal_codes);
        self.put(key, val)?; // forcibly fails
        Ok(())
    }
}

impl WritePostalCodesToRegionAndCity for FailingDbStub {
    fn write_postal_codes_to_region_and_city(
        &mut self, 
        region: &WorldRegion, 
        city: &CityName, 
        postal_codes: &BTreeSet<PostalCode>
    ) -> Result<(),DatabaseConstructionError> {
        let key = c2z_key(region, city);
        let val = compress_set_to_cbor(postal_codes);
        self.put(key, val)?;
        Ok(())
    }
}

// We only need this trait method:
impl WriteCitiesToRegionAndPostalCode for FailingDbStub {
    fn write_cities_to_region_and_postal_code(
        &mut self,
        region: &WorldRegion,
        postal_code: &PostalCode,
        cities: &BTreeSet<CityName>
    ) -> Result<(), DatabaseConstructionError> {
        let key = z2c_key(region, postal_code);
        let val = compress_set_to_cbor(cities);
        self.put(&key, val)?;
        Ok(())
    }
}

impl WriteCitiesToRegionAndStreet for FailingDbStub {
    fn write_cities_to_region_and_street(
        &mut self, 
        region: &WorldRegion, 
        street: &StreetName, 
        cities: &BTreeSet<CityName>
    ) -> Result<(),DatabaseConstructionError> {
        let key = s2c_key(region, street);
        let val = compress_set_to_cbor(cities);
        self.put(&key, val)?; // forcibly fails
        Ok(())
    }
}

impl WriteIndicesForRegion for FailingDbStub {
    fn write_indices_for_region(
        &mut self, 
        region: &WorldRegion, 
        indexes: &InMemoryIndexes
    ) -> Result<(), DatabaseConstructionError> {
        info!("Failing stub => calls each write method, but put always fails");
        // replicate the logic of the real function, or call it if you want a partial approach
        // We'll do the real logic:
        
        // if let Some(state_map) = indexes.postal_code_to_street_map_for_region(region) {
        //     for (postal_code, streets) in state_map {
        //         self.write_streets_to_region_and_postal_code(region, postal_code, streets)?;
        //     }
        // }
        // for (postal_code, cities) in indexes.postal_code_cities() {
        //     self.write_cities_to_region_and_postal_code(region, postal_code, cities)?;
        // }
        // ... etc for each

        // We'll skip the detail for brevity, but each call fails on put(...) => error
        Err(DatabaseConstructionError::SimulatedStoreFailure)
    }
}

// Minimal stub that updates the first street but fails on the second
pub struct FailingUpdateStub {
    pub(crate) calls: std::cell::RefCell<usize>,
}

impl FailingUpdateStub {
    pub fn new() -> Self {
        FailingUpdateStub {
            calls: std::cell::RefCell::new(0),
        }
    }
}

//--------------------------------------------------
// Define a custom DB type that simulates a load error.
pub struct FailingLoadDatabase {
    inner: Database,
}

impl FailingLoadDatabase {

    pub fn new() -> Result<Arc<Mutex<Self>>, WorldCityAndStreetDbBuilderError> 
    {
        let tmp = tempfile::TempDir::new().unwrap();
        Self::open(tmp.path())
    }
}

// Merge with `Database` or a minimal struct that the code calls:
impl OpenDatabaseAtPath for FailingLoadDatabase {

    fn open(path: impl AsRef<std::path::Path>) 
        -> Result<Arc<Mutex<Self>>, WorldCityAndStreetDbBuilderError> 
    {
        // Unwrap the Arc<Mutex<Database>> into a plain Database.
        let db_arc = Database::open(path).expect("Could not open DB");
        let db = Arc::try_unwrap(db_arc)
            .expect("Only one reference")
            .into_inner()
            .expect("Mutex poisoned");
        Ok(Arc::new(Mutex::new(Self { inner: db })))
    }
}

impl LoadExistingStreetRanges for FailingLoadDatabase {
    fn load_existing_street_ranges(
        &self,
        _r: &WorldRegion,
        _s: &StreetName,
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
        Err(DataAccessError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Simulated load error",
        )))
    }
}
impl StoreHouseNumberRanges for FailingLoadDatabase {
    fn store_house_number_ranges(
        &mut self,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) -> Result<(), DatabaseConstructionError> {
        self.inner.store_house_number_ranges(region, street, ranges)
    }
}

// We'll define a partial stub that fails on `load_existing_house_number_ranges`.
// Then confirm update_street_house_numbers => DataAccessError.
impl LoadHouseNumberRanges for FailingLoadDatabase {
    fn load_house_number_ranges(
        &self, 
        _region: &WorldRegion, 
        _street: &StreetName
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
        Err(DataAccessError::SimulatedReadError)
    }
}

//--------------------------------------------------
// Define a custom DB type that simulates a store error.
// If storing the final merged data fails => we see that error.
pub struct FailingStoreDatabase {
    inner: Database,
    // We'll hold a vector for existing data, so we can pass the load step
    existing: Vec<HouseNumberRange>,
}

impl FailingStoreDatabase {

    pub fn new() -> Result<Arc<Mutex<Self>>, WorldCityAndStreetDbBuilderError> 
    {
        let tmp = tempfile::TempDir::new().unwrap();
        Self::open(tmp.path())
    }

    pub fn set_existing(&mut self, existing: Vec<HouseNumberRange>) {
        self.existing = existing;
    }

    /// Allows us to store data directly into the underlying real DB
    /// without triggering the "SimulatedStoreFailure" override.
    pub fn store_ranges_via_inner(
        &mut self,
        region: &WorldRegion,
        street: &StreetName,
        ranges: &[HouseNumberRange],
    ) -> Result<(), DatabaseConstructionError> {
        // Use self.inner's store method, which is the real DB
        self.inner.store_house_number_ranges(region, street, ranges)
    }
}

impl OpenDatabaseAtPath for FailingStoreDatabase {

    fn open(path: impl AsRef<std::path::Path>) 
        -> Result<Arc<Mutex<Self>>, WorldCityAndStreetDbBuilderError> 
    {
        let db_arc = Database::open(path).expect("Could not open DB");

        let db = Arc::try_unwrap(db_arc)
            .expect("Only one reference")
            .into_inner()
            .expect("Mutex poisoned");

        Ok(Arc::new(Mutex::new(Self { inner: db, existing: vec![] })))
    }
}

impl LoadExistingStreetRanges for FailingStoreDatabase {
    fn load_existing_street_ranges(
        &self,
        region: &WorldRegion,
        street: &StreetName,
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
        self.inner.load_existing_street_ranges(region, street)
    }
}

impl StoreHouseNumberRanges for FailingStoreDatabase {
    fn store_house_number_ranges(
        &mut self,
        _region: &WorldRegion,
        _street: &StreetName,
        _ranges: &[HouseNumberRange],
    ) -> Result<(), DatabaseConstructionError> {
        // Return a simulated error using the Other variant.
        Err(DatabaseConstructionError::SimulatedStoreFailure)
    }
}

impl LoadHouseNumberRanges for FailingStoreDatabase {
    fn load_house_number_ranges(
        &self, 
        _region: &WorldRegion, 
        _street: &StreetName
    ) -> Result<Option<Vec<HouseNumberRange>>, DataAccessError> {
        Ok(Some(self.existing.clone()))
    }
}
