// ---------------- [ File: src/try_construct_postal_code.rs ]
crate::ix!();

/// Try to parse the `postcode_raw` as one or more codes separated by `';'`.
/// Returns `Ok(Some(PostalCode))` if at least one is valid, `Ok(None)` if `postcode_raw` is `None`,
/// or `Err(...)` if *all* subcodes are invalid.
#[tracing::instrument(level = "trace", skip_all)]
pub fn try_construct_postal_code(
    country: Country,
    postcode_raw: Option<&str>,
    element_id: i64,
) -> Result<Option<PostalCode>, IncompatibleOsmPbfElement> {
    use tracing::{debug, error};

    if let Some(raw_value) = postcode_raw {
        // Split on semicolons to handle multiple codes like "23060;23233"
        let candidates: Vec<&str> = raw_value.split(';').map(|s| s.trim()).collect();

        // Try each sub‐code; keep the first one that parses successfully
        for candidate in &candidates {
            if candidate.is_empty() {
                continue; // skip empty piece
            }
            match PostalCode::new(country, candidate) {
                Ok(pc) => {
                    debug!(
                        "try_construct_postal_code: sub-code '{}' is valid => {}",
                        candidate, pc.code()
                    );
                    return Ok(Some(pc));
                }
                Err(err) => {
                    error!(
                        "try_construct_postal_code: sub-code '{}' failed parse => {:?}",
                        candidate, err
                    );
                    // but keep trying other sub‐codes
                }
            }
        }

        // If we reach here, *none* of the sub‐codes were valid
        error!(
            "try_construct_postal_code: all sub‐codes invalid. Original='{}' (element_id={})",
            raw_value, element_id
        );
        return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::PostalCodeConstructionError(
                crate::PostalCodeConstructionError::InvalidFormat {
                    attempted_code: raw_value.to_string(),
                    attempted_country: Some(country),
                }
            )
        ));
    } else {
        // If no postcode tag at all, this is not an error; just "None".
        Ok(None)
    }
}

#[cfg(test)]
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
