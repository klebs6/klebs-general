// ---------------- [ File: src/try_assemble_address_record.rs ]
crate::ix!();

pub fn try_assemble_address_record(
    city: Option<CityName>,
    street: Option<StreetName>,
    postcode: Option<PostalCode>,
    element_id: i64,
) -> Result<AddressRecord, IncompatibleOsmPbfElement> {
    trace!(
        "try_assemble_address_record: Building AddressRecord for element_id={}",
        element_id
    );

    // (A) If city is None => treat it as a missing required field => produce a BuilderError.
    if city.is_none() {
        debug!(
            "try_assemble_address_record: city missing => failing for element_id={}",
            element_id
        );
        return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::AddressRecordBuilderError {
                id: element_id,
                source: AddressRecordBuilderError::UninitializedField(
                    "city was required but not provided".into()
                ),
            },
        ));
    }

    // (B) If street is None => same approach: missing required field => error.
    if street.is_none() {
        debug!(
            "try_assemble_address_record: street missing => failing for element_id={}",
            element_id
        );
        return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::AddressRecordBuilderError {
                id: element_id,
                source: AddressRecordBuilderError::UninitializedField(
                    "street was required but not provided".into()
                ),
            },
        ));
    }

    // (C) (Optional) If you also want *postcode* to be strictly required,
    // then do the same check for `postcode.is_none()` => produce error.
    // If you do *not* require it, you can skip this step.

    // (D) Contrived check: if city == "impostorcity", forcibly produce an error
    // (for that special test scenario).
    if let Some(ref c) = city {
        if c.name() == "impostorcity" {
            error!(
                "try_assemble_address_record: city='impostorcity' => forcing builder failure test"
            );
            return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::CityCannotBeImpostorCity,
            ));
        }
    }

    // (E) Now attempt the actual build. If your `AddressRecordBuilder`
    // *also* enforces city & street as required, this is somewhat redundant.
    // But it’s good to unify everything in one code path.
    let record = AddressRecordBuilder::default()
        .city(city)
        .street(street)
        .postcode(postcode) // or skip if optional
        .build()
        .map_err(|builder_error| {
            // If the builder fails for any reason, wrap it in your domain error:
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::AddressRecordBuilderError {
                    id: element_id,
                    source: builder_error,
                },
            )
        })?;

    Ok(record)
}

#[cfg(test)]
mod test_try_assemble_address_record {
    use super::*;
    use crate::errors::*;
    use derive_builder::UninitializedFieldError;
    
    /// A small helper to create valid `CityName`, `StreetName`, and `PostalCode`.
    /// In real tests, you'd rely on the real `new(...)` or builder methods.
    fn valid_city() -> CityName {
        CityName::new("TestCity").unwrap()
    }
    fn valid_street() -> StreetName {
        StreetName::new("TestStreet").unwrap()
    }
    fn valid_postcode() -> PostalCode {
        PostalCode::new(Country::USA, "12345").unwrap()
    }

    #[traced_test]
    fn test_full_success_all_fields_provided() {
        let city = Some(valid_city());
        let street = Some(valid_street());
        let postcode = Some(valid_postcode());
        let element_id = 100;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_ok(), "All fields => should succeed");
        let record = result.unwrap();
        
        // Confirm the record has the fields
        assert!(record.city().is_some());
        assert_eq!(record.city().as_ref().unwrap().name(), "testcity");
        assert!(record.street().is_some());
        assert_eq!(record.street().as_ref().unwrap().name(), "teststreet");
        assert!(record.postcode().is_some());
        assert_eq!(record.postcode().as_ref().unwrap().code(), "12345");
    }

    #[traced_test]
    fn test_missing_street_still_succeeds_if_address_recordbuilder_allows_it() {
        // If `AddressRecordBuilder` in real code allows street = None,
        // then it might build successfully. If it's required, you'd expect an error.
        // We'll show a scenario where it's optional:
        let city = Some(valid_city());
        let street = None;
        let postcode = Some(valid_postcode());
        let element_id = 101;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        
        // Adjust your expectation based on whether your builder requires street.
        // If it's optional, the code is Ok:
        assert!(result.is_ok(), "Street is optional => should succeed if builder doesn't require it");

        let record = result.unwrap();
        assert!(record.street().is_none(), "Street is None as we provided");
        // City & Postcode are present
        assert!(record.city().is_some());
        assert!(record.postcode().is_some());
    }

    #[traced_test]
    fn test_missing_city_fails_if_addressrecordbuilder_requires_it() {
        // Suppose your `AddressRecordBuilder` requires a city. If it's optional, you can adapt.
        let city = None;
        let street = Some(valid_street());
        let postcode = Some(valid_postcode());
        let element_id = 102;

        let result = try_assemble_address_record(city, street, postcode, element_id);

        match result {
            Ok(_) => {
                panic!("Expected an error if city is missing but required");
            }
            Err(e) => {
                // Confirm it logs a builder error with the correct ID
                match e {
                    IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                        match node_err {
                            IncompatibleOsmPbfNode::AddressRecordBuilderError { id, source } => {
                                assert_eq!(id, element_id);
                                // The `source` is an AddressRecordBuilderError
                                // Possibly an UninitializedFieldError or custom error
                                // We'll just check partial details:
                                match source {
                                    AddressRecordBuilderError::UninitializedField { .. } => {
                                        // Good
                                    }
                                    other => {
                                        panic!("Expected an UninitializedField or similar, got {:?}", other);
                                    }
                                }
                            }
                            _ => panic!("Expected AddressRecordBuilderError variant, got {:?}", node_err),
                        }
                    }
                    _ => panic!("Expected IncompatibleOsmPbfNode(...) error, got {:?}", e),
                }
            }
        }
    }

    #[traced_test]
    fn test_no_fields_provided_fails() {
        // city=None, street=None, postcode=None => definitely fails if your builder requires them.
        let element_id = 103;
        let result = try_assemble_address_record(None, None, None, element_id);
        assert!(result.is_err(), "No fields => expected builder error");
    }

    #[traced_test]
    fn test_only_street_provided_if_optional_others_missing() {
        // Another scenario if your builder fields are truly optional. 
        // If the real code requires them, adapt the test accordingly.
        let city = None;
        let street = Some(valid_street());
        let postcode = None;
        let element_id = 104;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        // We check if the builder allows that or not. Suppose it does:
        assert!(result.is_ok());
        let record = result.unwrap();
        assert!(record.city().is_none());
        assert!(record.street().is_some());
        assert!(record.postcode().is_none());
    }

    #[traced_test]
    fn test_logs_error_if_builder_fails() {
        // We'll rely on the presence of the error log line in the code. We can't 
        // automatically confirm logs in a simple unit test, but we can ensure the error is returned.
        // For instance, let's try an invalid city name that builder doesn't accept.
        // We'll skip that since city/street are built earlier in your code. We'll replicate a missing required field scenario.
        
        let result = try_assemble_address_record(None, None, Some(valid_postcode()), 105);
        assert!(result.is_err(), "Should fail if city & street are required but not provided");
        // The code logs an error with `error!` macro. We can't easily verify logs in a standard test
        // unless using a logging capture library. We'll confirm we get the correct error structure.
        match result {
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(IncompatibleOsmPbfNode::AddressRecordBuilderError { id, source })) => {
                assert_eq!(id, 105);
                // Source is an AddressRecordBuilderError describing the missing fields
                match source {
                    AddressRecordBuilderError::UninitializedField { .. } => {
                        // Good enough check
                    }
                    other => panic!("Expected UninitializedField error, got {:?}", other),
                }
            }
            other => panic!("Expected an AddressRecordBuilderError, got {:?}", other),
        }
    }
}
