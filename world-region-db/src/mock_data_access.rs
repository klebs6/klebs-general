// ---------------- [ File: src/mock_data_access.rs ]
// We'll create minimal scaffolding to simulate valid/invalid addresses 
// and parse errors, as well as a mock DataAccess that can validate them.
//
// In real code, you'd likely use your existing `DataAccess` or partial mocks 
// to control whether addresses pass/fail validation.
crate::ix!();

#[derive(Clone, Default)]
pub struct MockDataAccess {
    // 1) Addresses that should fail no matter what
    invalid_addresses: Arc<Mutex<Vec<WorldAddress>>>,
    // 2) A map from (regionAbbrev, postalCode) -> set of valid cityNames
    postal_to_city_map: Arc<Mutex<HashMap<(String, String), HashSet<String>>>>,
}

impl MockDataAccess {
    pub fn new() -> Self {
        Self {
            invalid_addresses: Arc::new(Mutex::new(Vec::new())),
            postal_to_city_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// If we push an address in here, we always fail it in `validate_with`.
    pub fn invalidate_address(&mut self, addr: WorldAddress) {
        let mut lock = self.invalid_addresses.lock().unwrap();
        lock.push(addr);
    }

    /// This is the method your test calls.  It records a set of city names that
    /// are valid for `(regionAbbrev, postalCode)`.
    pub fn add_postal_code_city_list(
        &mut self,
        region_abbrev: &str,
        postal_code: &str,
        cities: Vec<&str>,
    ) {
        let mut map_lock = self.postal_to_city_map.lock().unwrap();
        let key = (region_abbrev.to_string(), postal_code.to_string());
        let entry = map_lock.entry(key).or_insert_with(HashSet::new);
        for c in cities {
            entry.insert(c.to_string());
        }
    }
}

// Then, in `validate_with`, we check both “invalid_addresses” *and* the city–postal map.
impl ValidateWith<MockDataAccess> for WorldAddress {
    type Error = InvalidWorldAddress;

    fn validate_with(&self, mock: &MockDataAccess) -> Result<(), Self::Error> {
        // 1) If this address is in the “invalid_addresses” list => fail
        let lock = mock.invalid_addresses.lock().unwrap();
        if lock.contains(self) {
            return Err(InvalidWorldAddress::StreetNotFoundForPostalCodeInRegion {
                street: self.street().clone(),
                region: *self.region(),
                postal_code: self.postal_code().clone(),
            });
        }
        drop(lock);

        // 2) Otherwise, check if city is valid for (region, postalCode).
        //    We'll assume your `WorldAddress` has region().abbrev() or similar:
        let region_abbrev = self.region().abbreviation();
        let pc = self.postal_code().code();
        let city = self.city().name();

        let map = mock.postal_to_city_map.lock().unwrap();
        let key = (region_abbrev.to_string(), pc.to_string());
        match map.get(&key) {
            Some(valid_cities) => {
                if valid_cities.contains(city) {
                    // Great, city is recognized => pass
                    Ok(())
                } else {
                    // City not recognized => fail
                    Err(InvalidWorldAddress::CityNotFoundForPostalCodeInRegion {
                        city: self.city().clone(),
                        region: *self.region(),
                        postal_code: self.postal_code().clone(),
                    })
                }
            }
            None => {
                // No entry at all for this region+postal => fail
                Err(InvalidWorldAddress::PostalCodeToCityKeyNotFoundForRegion {
                    z2c_key: format!("Z2C:{:?}:{}", self.region(), pc),
                    region: *self.region(),
                    postal_code: self.postal_code().clone(),
                })
            }
        }
    }
}

// We'll also implement a trivial conversion from the real DataAccess to the MockDataAccess trait,
// just so the function signature remains the same. But for real usage, you might mock DataAccess differently.
impl<I:StorageInterface> From<MockDataAccess> for DataAccess<I> {
    fn from(_mock: MockDataAccess) -> Self {
        // Real DataAccess is presumably more complex. For the sake of this test,
        // we won't fully convert. In reality you'd reorganize the code or 
        // pass MockDataAccess directly into `process_and_validate_addresses` 
        // by making the function generically accept anything that implements 
        // the needed trait.
        unimplemented!("A direct From<MockDataAccess> for DataAccess is not typically needed. This is just a placeholder if your real code requires it.");
    }
}

pub fn create_data_access_for_mock<I:StorageInterface>(_mock: &MockDataAccess) -> DataAccess<I> {
    // Spin up a temporary DB in a temp directory or memory:
    let tmp = tempfile::TempDir::new().unwrap();
    let db = I::open(tmp.path()).expect("Failed to open RocksDB in temp dir");
    DataAccess::with_db(db)
}

/// Opens a temporary `Database` for testing.
pub fn create_temp_db<I:StorageInterface>() -> (Arc<Mutex<I>>, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db = I::open(temp_dir.path()).expect("Failed to open database in temp dir");
    (db, temp_dir)
}

/// Creates a DataAccess from a newly opened DB in a temp directory.
pub fn create_data_access<I:StorageInterface>() -> (DataAccess<I>, Arc<Mutex<I>>, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db = I::open(temp_dir.path()).expect("Failed to open DB");
    let db_arc = db.clone();
    let data_access = DataAccess::with_db(db);
    (data_access, db_arc, temp_dir)
}
