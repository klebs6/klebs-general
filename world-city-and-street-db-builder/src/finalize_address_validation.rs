crate::ix!();

/// Inspects whether all addresses were valid, returning a success or a
/// `NotAllAddressesValidatedSuccessfully` error.
pub fn finalize_address_validation(all_valid: bool) -> Result<(), WorldCityAndStreetDbBuilderError> {
    trace!("finalize_address_validation: all_valid={}", all_valid);
    if !all_valid {
        warn!("finalize_address_validation: Not all addresses validated successfully");
        Err(WorldCityAndStreetDbBuilderError::NotAllAddressesValidatedSuccessfully)
    } else {
        info!("finalize_address_validation: all addresses validated successfully");
        Ok(())
    }
}
