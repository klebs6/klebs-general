// ---------------- [ File: src/process_and_validate_addresses.rs ]
crate::ix!();

/// Consumes the address iterator, validating each [`WorldAddress`].
/// Returns `Ok(true)` if all addresses are valid, `Ok(false)` otherwise.
pub fn process_and_validate_addresses<I>(
    address_iter: I,
    data_access: &DataAccess
) -> Result<bool, WorldCityAndStreetDbBuilderError>
where
    I: Iterator<Item = Result<WorldAddress, OsmPbfParseError>>,
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
