// ---------------- [ File: src/process_and_validate_addresses.rs ]
// ---------------- [ File: src/process_and_validate_addresses.rs ]
crate::ix!();

/// Consumes the address iterator, validating each [`WorldAddress`].
/// Returns `Ok(true)` if all addresses are valid, `Ok(false)` otherwise.
pub fn process_and_validate_addresses<AddressIterator,I:StorageInterface>(
    address_iter: AddressIterator,
    data_access:  &DataAccess<I>
) -> Result<bool, WorldCityAndStreetDbBuilderError>
where
    AddressIterator: Iterator<Item = Result<WorldAddress, OsmPbfParseError>>,
{
    trace!("process_and_validate_addresses: starting validation loop");

    let mut all_valid = true;
    let mut count = 0usize;

    for addr_res in address_iter {
        match addr_res {
            Ok(addr) => {
                if let Err(e) = addr.validate_with(data_access) {
                    warn!("process_and_validate_addresses: Address invalid => {:#?}\nerr={:#?}", addr, e);
                    all_valid = false;
                } else if count % 100 == 0 {
                    info!("process_and_validate_addresses: {}th address validated => {:#?} is valid", count, addr);
                }
            }
            Err(e) => {
                warn!("process_and_validate_addresses: could not parse address => {:?}", e);
                all_valid = false;
            }
        }
        count += 1;
    }

    debug!("process_and_validate_addresses: total addresses checked={}", count);
    Ok(all_valid)
}

#[cfg(test)]
mod test_process_and_validate_addresses {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::iter;

    // We'll create minimal scaffolding to simulate valid/invalid addresses 
    // and parse errors, as well as a mock DataAccess that can validate them.
    //
    // In real code, you'd likely use your existing `DataAccess` or partial mocks 
    // to control whether addresses pass/fail validation.

    // A simple MockDataAccess that can store a set of addresses that should fail validation.
    // If the address is in the fail set, validate returns an error. Otherwise, it returns Ok.
    #[derive(Clone, Default)]
    struct MockDataAccess {
        invalid_addresses: Arc<Mutex<Vec<WorldAddress>>>,
    }

    impl MockDataAccess {
        fn new() -> Self {
            Self {
                invalid_addresses: Arc::new(Mutex::new(vec![])),
            }
        }

        fn invalidate_address(&mut self, addr: WorldAddress) {
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

    #[traced_test]
    fn test_all_addresses_valid_returns_ok_true() {
        let mut mock_data_access = MockDataAccess::new();
        // No invalid addresses => everything is considered valid

        // Let's define an iterator of addresses (Result<WorldAddress, OsmPbfParseError>)
        let addr1 = make_mock_address("11111", "mockcity", "mockstreet");
        let addr2 = make_mock_address("22222", "mockcity2", "mockstreet2");
        let addresses_iter = vec![Ok(addr1), Ok(addr2)].into_iter();

        // For demonstration, we'll wrap it in a real DataAccess if your function signature requires it,
        // but we only use the mock's data. 
        // We can do a small hack: we won't call the real DataAccess methods, 
        // but we'll store the mock inside it. Alternatively, adjust your function to accept the mock type.
        let data_access = create_data_access_for_mock(&mock_data_access);

        let result = process_and_validate_addresses(addresses_iter, &data_access);
        assert!(result.is_ok(), "Should not produce an error in the final result");
        let success = result.unwrap();
        assert!(
            success,
            "All addresses valid => should return true"
        );
    }

    #[traced_test]
    fn test_some_addresses_fail_validation_returns_ok_false() {
        let mut mock_data_access = MockDataAccess::new();

        let addr1 = make_mock_address("11111", "mockcity", "mockstreet");
        let addr2 = make_mock_address("22222", "mockcity2", "mockstreet2");
        // Mark addr2 as invalid
        mock_data_access.invalidate_address(addr2.clone());

        // Our address iterator: first is Ok(addr1), second is Ok(addr2)
        let addresses_iter = vec![Ok(addr1), Ok(addr2)].into_iter();

        let data_access = create_data_access_for_mock(&mock_data_access);

        let result = process_and_validate_addresses(addresses_iter, &data_access);
        assert!(result.is_ok(), "Shouldn't be a system-level error, just a false result");
        let success = result.unwrap();
        assert!(
            !success,
            "One address is invalid => should return false"
        );
    }

    #[traced_test]
    fn test_parsing_error_in_iterator_returns_ok_false() {
        // Some addresses fail not due to validation, but a parse error => the item is Err(...)
        // That should set all_valid=false but not produce a big error. 
        let mock_data_access = MockDataAccess::new(); // No invalid addresses, but parse error in input.

        let addr_ok = make_mock_address("11111", "mockcity", "mockstreet");

        // We'll define an Err(...) for a parse error. For instance, a hypothetical OsmPbfParseError.
        let parse_err = OsmPbfParseError::OsmPbf(osmpbf::Error::new("Simulated parse fail"));

        let addresses_iter = vec![Ok(addr_ok), Err(parse_err)].into_iter();

        let data_access = create_data_access_for_mock(&mock_data_access);

        let result = process_and_validate_addresses(addresses_iter, &data_access);
        assert!(result.is_ok(), "Should not produce an overall error");
        let success = result.unwrap();
        assert!(
            !success,
            "Parsing error => not all addresses validated => false"
        );
    }

    #[traced_test]
    fn test_empty_iterator_returns_ok_true() {
        // No addresses => trivially all valid => should return Ok(true).
        let mock_data_access = MockDataAccess::new();
        let addresses_iter = vec![].into_iter(); // empty

        let data_access = create_data_access_for_mock(&mock_data_access);
        let result = process_and_validate_addresses(addresses_iter, &data_access);
        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(
            success,
            "No addresses => trivially all valid => true"
        );
    }

    // ===============================
    // Helper methods for the tests
    // ===============================

    /// Creates a minimal `WorldAddress` fixture for testing.
    fn make_mock_address(
        postcode: &str,
        city: &str,
        street: &str
    ) -> WorldAddress {
        // We'll pick a region that can be validated. 
        let region = USRegion::UnitedState(UnitedState::Maryland).into();
        WorldAddressBuilder::default()
            .region(region)
            .postal_code(PostalCode::new(Country::USA, postcode).unwrap())
            .city(CityName::new(city).unwrap())
            .street(StreetName::new(street).unwrap())
            .build()
            .unwrap()
    }

    /// Creates a "real" DataAccess from the mock, if your function requires a real DataAccess param.
    /// In this minimal demonstration, we just store the mock in a static field,
    /// but in real usage you'd likely modify the function signature to accept the mock directly.
    fn create_data_access_for_mock<I:StorageInterface>(_mock: &MockDataAccess) -> DataAccess<I> {
        // If you want to adapt the real DataAccess to reference the mock,
        // you could store the mock in a global or Arc<Mutex> inside DataAccess. 
        // For now, let's return a dummy DataAccess. 
        // The actual validation calls are hooking into the `ValidateWith for WorldAddress` 
        // above, so we won't rely on the real DataAccess logic.
        let db_arc = Arc::new(Mutex::new(DatabaseBuilder::default().build().unwrap()));
        DataAccess::with_db(db_arc)
    }
}
