// We'll create minimal scaffolding to simulate valid/invalid addresses 
// and parse errors, as well as a mock DataAccess that can validate them.
//
// In real code, you'd likely use your existing `DataAccess` or partial mocks 
// to control whether addresses pass/fail validation.
crate::ix!();

// A simple MockDataAccess that can store a set of addresses that should fail validation.
// If the address is in the fail set, validate returns an error. Otherwise, it returns Ok.
#[derive(Clone, Default)]
pub struct MockDataAccess {
    invalid_addresses: Arc<Mutex<Vec<WorldAddress>>>,
}

impl MockDataAccess {
    pub fn new() -> Self {
        Self {
            invalid_addresses: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn invalidate_address(&mut self, addr: WorldAddress) {
        let mut lock = self.invalid_addresses.lock().unwrap();
        lock.push(addr);
    }
}

impl ValidateWith<MockDataAccess> for WorldAddress {
    type Error = InvalidWorldAddress;

    fn validate_with(&self, validator: &MockDataAccess) -> Result<(), Self::Error> {
        let lock = validator.invalid_addresses.lock().unwrap();
        if lock.contains(self) {
            Err(InvalidWorldAddress::StreetNotFoundForPostalCodeInRegion {
                street: self.street().clone(),
                region: *self.region(),
                postal_code: self.postal_code().clone(),
            })
        } else {
            Ok(())
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
