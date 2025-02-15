// ---------------- [ File: src/try_construct_street_name.rs ]
crate::ix!();

/// Attempts to create a `StreetName` from a string (if present). Returns an error
/// if construction fails.
pub fn try_construct_street_name(
    street_raw: Option<&str>,
    element_id: i64,
) -> Result<Option<StreetName>, IncompatibleOsmPbfElement> {
    if let Some(raw_value) = street_raw {
        trace!("try_construct_street_name: Parsing street for element_id={}", element_id);
        match StreetName::new(raw_value) {
            Ok(street) => Ok(Some(street)),
            Err(e) => {
                error!("try_construct_street_name: StreetName construction error for element_id={}: {:?}", element_id, e);
                Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                    IncompatibleOsmPbfNode::StreetNameConstructionError(e),
                ))
            }
        }
    } else {
        debug!("try_construct_street_name: No street tag for element_id={}", element_id);
        Ok(None)
    }
}

#[cfg(test)]
mod test_try_construct_street_name {
    use super::*;
    use crate::errors::*;
    use tracing::{trace, debug, error};

    #[traced_test]
    fn test_none_input_returns_ok_none() {
        let element_id = 10;
        let result = try_construct_street_name(None, element_id);
        assert!(result.is_ok(), "No street => Ok(None)");
        assert!(result.unwrap().is_none());
    }

    #[traced_test]
    fn test_valid_street_returns_some() {
        let element_id = 11;
        let result = try_construct_street_name(Some("Main Street"), element_id);
        assert!(result.is_ok(), "A valid street name should succeed");
        let street = result.unwrap().expect("Expected Some(StreetName)");
        // StreetName::new("Main Street") typically normalizes to "main street"
        assert_eq!(street.name(), "main street",
            "Normalization should have occurred if your StreetName does so"
        );
    }

    #[traced_test]
    fn test_empty_street_fails() {
        let element_id = 12;
        let result = try_construct_street_name(Some(""), element_id);
        match result {
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::StreetNameConstructionError(e)
            )) => {
                // The error might be StreetNameConstructionError::InvalidName { attempted_name } 
                // or something similar, depending on your definition.
                match e {
                    StreetNameConstructionError::InvalidName { attempted_name } => {
                        assert!(attempted_name.is_empty());
                    }
                    _ => panic!("Expected InvalidName variant, got {:?}", e),
                }
            }
            other => panic!("Expected StreetNameConstructionError, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_whitespace_street_fails() {
        let element_id = 13;
        let result = try_construct_street_name(Some("   "), element_id);
        match result {
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::StreetNameConstructionError(_)
            )) => {
                // Good; likely StreetNameConstructionError::InvalidName
            }
            other => panic!("Expected a StreetNameConstructionError, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_debug_logging_when_streetname_fails() {
        // The function logs an error with the `error!` macro if street construction fails.
        // We can't directly verify logs in a standard test, but we confirm the error is returned.
        let element_id = 14;
        let result = try_construct_street_name(Some("***InvalidStreet***"), element_id);
        assert!(result.is_err(), "Expecting an error if your StreetName logic disallows such strings");
    }
}
