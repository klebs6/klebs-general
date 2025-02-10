// ---------------- [ File: src/try_construct_postal_code.rs ]
// ---------------- [ File: src/try_construct_postal_code.rs ]
crate::ix!();

/// Attempts to create a `PostalCode` from a string (if present). Returns an error
/// if construction fails.
pub fn try_construct_postal_code(
    country: Country,
    postcode_raw: Option<&str>,
    element_id: i64,
) -> Result<Option<PostalCode>, IncompatibleOsmPbfElement> {
    if let Some(raw_value) = postcode_raw {
        trace!("try_construct_postal_code: Parsing postcode for element_id={}", element_id);
        match PostalCode::new(country, raw_value) {
            Ok(pc) => Ok(Some(pc)),
            Err(e) => {
                error!("try_construct_postal_code: PostalCode construction error for element_id={}: {:?}", element_id, e);
                Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                    IncompatibleOsmPbfNode::PostalCodeConstructionError(e),
                ))
            }
        }
    } else {
        debug!("try_construct_postal_code: No postcode tag for element_id={}", element_id);
        Ok(None)
    }
}

#[cfg(test)]
#[disable]
mod test_try_construct_postal_code {
    use super::*;
    use crate::errors::*; // Adjust if PostalCodeConstructionError or IncompatibleOsmPbfElement live elsewhere

    fn test_country() -> Country {
        Country::USA // or whichever is appropriate
    }

    #[traced_test]
    fn test_none_input_returns_ok_none() {
        let element_id = 10;
        let result = try_construct_postal_code(test_country(), None, element_id);
        assert!(result.is_ok(), "No postcode => Ok(None)");
        assert!(result.unwrap().is_none());
    }

    #[traced_test]
    fn test_valid_postcode_returns_ok_some() {
        let element_id = 11;
        let result = try_construct_postal_code(test_country(), Some("12345"), element_id);
        assert!(result.is_ok(), "Valid US ZIP");
        let pc = result.unwrap().expect("Should be Some(...)");
        assert_eq!(pc.code(), "12345", "Should match the input");
    }

    #[traced_test]
    fn test_empty_postcode_fails() {
        let element_id = 12;
        let result = try_construct_postal_code(test_country(), Some(""), element_id);
        match result {
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::PostalCodeConstructionError(e)
            )) => {
                // e might be a variant of your postal code error, e.g. InvalidPostalCode
            }
            other => panic!("Expected PostalCodeConstructionError, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_invalid_postcode_fails() {
        let element_id = 13;
        // Suppose "ABCD" is invalid for the country
        let result = try_construct_postal_code(test_country(), Some("ABCDE"), element_id);
        match result {
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::PostalCodeConstructionError(e)
            )) => {
                // Good
            }
            other => panic!("Expected PostalCodeConstructionError, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_debug_logging_on_error() {
        // The function logs an error if postal code construction fails.
        // We can't confirm logs in a basic test; we simply confirm the error is returned.
        let element_id = 14;
        let result = try_construct_postal_code(test_country(), Some("???"), element_id);
        assert!(result.is_err(), "Invalid => error, logs an error!");
    }
}
