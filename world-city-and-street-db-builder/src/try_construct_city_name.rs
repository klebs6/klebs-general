// ---------------- [ File: src/try_construct_city_name.rs ]
crate::ix!();

/// Attempts to create a `CityName` from a string (if present). Returns an error
/// if construction fails.
pub fn try_construct_city_name(
    city_raw: Option<&str>,
    element_id: i64,
) -> Result<Option<CityName>, IncompatibleOsmPbfElement> {
    if let Some(raw_value) = city_raw {
        trace!("try_construct_city_name: Parsing city for element_id={}", element_id);
        match CityName::new(raw_value) {
            Ok(city) => Ok(Some(city)),
            Err(e) => {
                error!("try_construct_city_name: CityName construction error for element_id={}: {:?}", element_id, e);
                Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                    IncompatibleOsmPbfNode::CityNameConstructionError(e),
                ))
            }
        }
    } else {
        debug!("try_construct_city_name: No city tag for element_id={}", element_id);
        Ok(None)
    }
}

#[cfg(test)]
#[disable]
mod test_try_construct_city_name {
    use super::*;
    use crate::errors::*; // Adjust if your error types or CityNameConstructionError live elsewhere

    #[test]
    fn test_none_input_returns_ok_none() {
        let element_id = 1;
        let result = try_construct_city_name(None, element_id);
        assert!(result.is_ok(), "No city => Ok(...)");
        let city_opt = result.unwrap();
        assert!(city_opt.is_none(), "Expected None city");
    }

    #[test]
    fn test_valid_city_returns_ok_some() {
        let element_id = 2;
        let result = try_construct_city_name(Some("Baltimore"), element_id);
        assert!(result.is_ok(), "Valid city => Ok(Some(...))");
        let city_opt = result.unwrap().expect("Should be Some(...)");
        assert_eq!(city_opt.name(), "baltimore",
            "Expected normalized 'baltimore' from 'Baltimore'");
    }

    #[test]
    fn test_empty_city_name_fails() {
        let element_id = 3;
        let result = try_construct_city_name(Some(""), element_id);
        match result {
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::CityNameConstructionError(e)
            )) => {
                // This is expected if CityName::new("") is invalid
                match e {
                    CityNameConstructionError::InvalidName { .. } => {
                        // Good
                    }
                    other => panic!("Expected CityNameConstructionError::InvalidName, got {:?}", other),
                }
            }
            other => panic!("Expected CityNameConstructionError, got {:?}", other),
        }
    }

    #[test]
    fn test_whitespace_city_name_fails_if_disallowed() {
        let element_id = 4;
        let result = try_construct_city_name(Some("   "), element_id);
        match result {
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::CityNameConstructionError(e)
            )) => {
                // Likely "InvalidName"
                match e {
                    CityNameConstructionError::InvalidName { attempted_name } => {
                        assert!(attempted_name.trim().is_empty(), 
                            "Expected '   ' to be considered invalid");
                    }
                    other => panic!("Expected CityNameConstructionError::InvalidName, got {:?}", other),
                }
            }
            Ok(opt) => panic!("Should fail on whitespace city, got {:?}", opt),
            other => panic!("Expected CityNameConstructionError, got {:?}", other),
        }
    }

    #[test]
    fn test_debug_logging_when_cityname_fails() {
        // The code logs an error with `error!` macro if city construction fails. 
        // We can't directly verify logs without a logging capture. 
        // We'll just ensure the error is returned.
        let element_id = 5;
        let result = try_construct_city_name(Some("???invalid???"), element_id);
        assert!(result.is_err(), "Expecting an error for this city name if your logic disallows it.");
    }
}
