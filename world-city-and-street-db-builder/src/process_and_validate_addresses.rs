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
        let data_access = create_data_access_for_mock::<Database>(&mock_data_access);

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

        let data_access = create_data_access_for_mock::<Database>(&mock_data_access);

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
        let parse_err = OsmPbfParseError::SimulatedParseFail;

        let addresses_iter = vec![Ok(addr_ok), Err(parse_err)].into_iter();

        let data_access = create_data_access_for_mock::<Database>(&mock_data_access);

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

        let data_access = create_data_access_for_mock::<Database>(&mock_data_access);
        let result = process_and_validate_addresses(addresses_iter, &data_access);
        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(
            success,
            "No addresses => trivially all valid => true"
        );
    }
}
