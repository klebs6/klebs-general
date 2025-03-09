// ---------------- [ File: src/try_construct_multiple_postal_codes.rs ]
crate::ix!();

pub fn try_construct_multi_postal_codes(
    country: Country,
    raw_value: &str,
    element_id: i64,
) -> Result<Vec<PostalCode>, IncompatibleOsmPbfElement> {
    use tracing::{trace, debug, error};
    use crate::{PostalCode, OsmPbfParseError, IncompatibleOsmPbfElement, IncompatibleOsmPbfNode};

    trace!(
        "try_construct_multi_postal_codes: attempting to parse '{}' with semicolons (element_id={})",
        raw_value,
        element_id
    );

    // Split on semicolons (some data uses e.g. "23060;23233")
    let candidates: Vec<&str> = raw_value.split(';').map(|s| s.trim()).collect();
    if candidates.is_empty() {
        error!(
            "try_construct_multi_postal_codes: no substring found in '{}' (element_id={})",
            raw_value,
            element_id
        );
        return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::PostalCodeConstructionError(
                crate::PostalCodeConstructionError::InvalidFormat {
                    attempted_code: raw_value.to_string(),
                    attempted_country: Some(country),
                }
            )
        ));
    }

    let mut parsed_codes = Vec::new();
    for candidate in candidates {
        if candidate.is_empty() {
            // skip empty
            continue;
        }
        match PostalCode::new(country, candidate) {
            Ok(pc) => {
                debug!(
                    "try_construct_multi_postal_codes: parsed valid code '{}' => {:?}",
                    candidate, pc
                );
                parsed_codes.push(pc);
            }
            Err(e) => {
                // We log it, but keep trying to parse the others
                error!(
                    "try_construct_multi_postal_codes: error parsing '{}' => {:?}",
                    candidate, e
                );
            }
        }
    }

    if parsed_codes.is_empty() {
        // If *none* of the subcodes were valid, return an error
        Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::PostalCodeConstructionError(
                crate::PostalCodeConstructionError::InvalidFormat {
                    attempted_code: raw_value.to_string(),
                    attempted_country: Some(country),
                }
            )
        ))
    } else {
        // At least one was good
        Ok(parsed_codes)
    }
}

#[cfg(test)]
mod test_multi_postal_codes {
    use super::*;
    use crate::Country;

    #[test]
    fn test_multi_postal_codes_success() {
        // Suppose "23060;23233" => we parse two codes
        let country = Country::USA;
        let result = try_construct_multi_postal_codes(country, "23060;23233", 12345).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].code(), "23060");
        assert_eq!(result[1].code(), "23233");
    }

    #[test]
    fn test_multi_postal_codes_some_invalid() {
        // Contains one valid code "30000" and one invalid "12"
        let country = Country::USA;
        let result = try_construct_multi_postal_codes(country, "30000 ; 12", 789);
        // "12" might fail your validation, but "30000" is presumably valid
        // We only fail if *all* are invalid, so this should succeed with [30000].
        let codes = result.expect("expected at least one valid code");
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].code(), "30000");
    }

    #[test]
    fn test_multi_postal_codes_all_fail() {
        // "abc;123" might be invalid for your logic
        let country = Country::USA;
        let result = try_construct_multi_postal_codes(country, "abc;  12", 555);
        assert!(result.is_err(), "No valid codes => should error");
    }

    #[test]
    fn test_multi_postal_codes_empty() {
        // Edge case: empty string or only semicolons
        let country = Country::USA;
        let result = try_construct_multi_postal_codes(country, ";", 999);
        assert!(result.is_err());
    }
}
