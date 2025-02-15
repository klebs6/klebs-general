// ---------------- [ File: src/finalize_address_validation.rs ]
// ---------------- [ File: src/finalize_address_validation.rs ]
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

#[cfg(test)]
mod finalize_address_validation_tests {
    use super::*;

    #[traced_test]
    fn test_finalize_address_validation_success() {
        // When all_valid is true, we expect Ok(())
        let result = finalize_address_validation(true);
        assert!(result.is_ok(), "Expected Ok(()) when all_valid is true");
    }

    #[traced_test]
    fn test_finalize_address_validation_failure() {
        // When all_valid is false, we expect an error.
        let result = finalize_address_validation(false);
        assert!(result.is_err(), "Expected an error when all_valid is false");
        match result.err().unwrap() {
            WorldCityAndStreetDbBuilderError::NotAllAddressesValidatedSuccessfully => {
                // Expected variant.
            }
            other => {
                panic!("Expected NotAllAddressesValidatedSuccessfully error, got: {:?}", other);
            }
        }
    }
}
