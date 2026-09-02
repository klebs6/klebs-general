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
    use std::collections::BTreeSet;

    #[traced_test]
    fn test_all_addresses_valid_returns_ok_true() {
        // 1) Create DataAccess + DB in a temp dir
        let (data_access, _db_arc, _temp_dir) = create_data_access::<Database>();
        let region = USRegion::UnitedState(UnitedState::Florida).into();

        // 2) Lock the DB and insert all needed sets
        {
            let mut db_guard = data_access.db().lock().unwrap();

            // (a) city-to-postal: Z2C:FL:11111 => {mockcity}, Z2C:FL:22222 => {mockcity2}
            let mut city_set1 = BTreeSet::new();
            city_set1.insert(CityName::new("mockcity").unwrap());
            db_guard.put(
                z2c_key(&region, &PostalCode::new(Country::USA, "11111").unwrap()),
                compress_set_to_cbor(&city_set1),
            ).unwrap();

            let mut city_set2 = BTreeSet::new();
            city_set2.insert(CityName::new("mockcity2").unwrap());
            db_guard.put(
                z2c_key(&region, &PostalCode::new(Country::USA, "22222").unwrap()),
                compress_set_to_cbor(&city_set2),
            ).unwrap();

            // (b) postal-to-street: S:FL:11111 => {mockstreet}, S:FL:22222 => {mockstreet2}
            let mut st_set1 = BTreeSet::new();
            st_set1.insert(StreetName::new("mockstreet").unwrap());
            db_guard.put(
                s_key(&region, &PostalCode::new(Country::USA, "11111").unwrap()),
                compress_set_to_cbor(&st_set1),
            ).unwrap();

            let mut st_set2 = BTreeSet::new();
            st_set2.insert(StreetName::new("mockstreet2").unwrap());
            db_guard.put(
                s_key(&region, &PostalCode::new(Country::USA, "22222").unwrap()),
                compress_set_to_cbor(&st_set2),
            ).unwrap();

            // (c) city-to-street: C2S:FL:mockcity => {mockstreet}, C2S:FL:mockcity2 => {mockstreet2}
            let mut city2street1 = BTreeSet::new();
            city2street1.insert(StreetName::new("mockstreet").unwrap());
            db_guard.put(
                c2s_key(&region, &CityName::new("mockcity").unwrap()),
                compress_set_to_cbor(&city2street1),
            ).unwrap();

            let mut city2street2 = BTreeSet::new();
            city2street2.insert(StreetName::new("mockstreet2").unwrap());
            db_guard.put(
                c2s_key(&region, &CityName::new("mockcity2").unwrap()),
                compress_set_to_cbor(&city2street2),
            ).unwrap();
        }

        // 3) Make our addresses, each with city, postal_code, street
        let addr1 = make_mock_address("11111", "mockcity",  "mockstreet");
        let addr2 = make_mock_address("22222", "mockcity2", "mockstreet2");
        let addresses_iter = vec![Ok(addr1), Ok(addr2)].into_iter();

        // 4) Run the validation routine
        let result = process_and_validate_addresses(addresses_iter, &data_access);

        // 5) Confirm that everything is valid => returns Ok(true)
        assert!(result.is_ok());
        let all_valid = result.unwrap();
        assert!(all_valid, "All addresses valid => should return true");
    }

    #[traced_test]
    fn test_some_addresses_fail_validation_returns_ok_false() {
        let mut mock_data_access = MockDataAccess::new();

        let addr1 = make_mock_address("11111", "mockcity", "mockstreet");
        let addr2 = make_mock_address("22222", "mockcity2", "mockstreet2");
        // Mark addr2 as invalid in our mock data
        mock_data_access.invalidate_address(addr2.clone());

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
        let mock_data_access = MockDataAccess::new();

        let addr_ok = make_mock_address("11111", "mockcity", "mockstreet");
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
