// ---------------- [ File: src/try_assemble_address_record.rs ]
crate::ix!();

/// Builds an AddressRecord from the optional city/street/postcode. 
/// Now relaxed so that any combination of 2 or 3 fields is “valid”, 
/// city-only or street-only triggers a warning, zip-only triggers an error,
/// and none triggers an error.
pub fn try_assemble_address_record(
    city:       Option<CityName>,
    street:     Option<StreetName>,
    postcode:   Option<PostalCode>,
    element_id: i64,

) -> Result<Option<AddressRecord>, IncompatibleOsmPbfElement> 
{
    // Count how many fields are actually present.
    let present_count = (city.is_some() as u8)
        + (street.is_some() as u8)
        + (postcode.is_some() as u8);

    if let Some(ref c) = city {
        // still do the "impostorcity" check if relevant
        if c.name() == "impostorcity" {
            error!(
                "try_assemble_address_record_relaxed: city='impostorcity' => forcing builder failure test (element_id={})",
                element_id
            );
            return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::CityCannotBeImpostorCity,
            ));
        }
    }

    match present_count {
        3 | 2 => {
            // Normal build if it has at least 2 fields:
            let record = AddressRecordBuilder::default()
                .city(city)
                .street(street)
                .postcode(postcode)
                .build()
                .map_err(|builder_err| {
                    IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                        IncompatibleOsmPbfNode::AddressRecordBuilderError {
                            id: element_id,
                            source: builder_err,
                        },
                    )
                })?;
            debug!(
                "try_assemble_address_record_relaxed: built a record with {} fields => {:?}",
                present_count, record
            );
            Ok(Some(record))
        }
        1 => {
            // Exactly one field => check which:
            match (city, street, postcode) {
                (Some(city_only), None, None) => {
                    // City only => warn, but build partial
                    warn!(
                        "try_assemble_address_record_relaxed: city-only (element_id={}), building partial record",
                        element_id
                    );
                    let record = AddressRecordBuilder::default()
                        .city(Some(city_only))
                        .build()
                        .map_err(|builder_err| {
                            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                                IncompatibleOsmPbfNode::AddressRecordBuilderError {
                                    id: element_id,
                                    source: builder_err,
                                },
                            )
                        })?;
                    Ok(Some(record))
                }
                (None, Some(street_only), None) => {
                    // Street only => warn, but build partial
                    warn!(
                        "try_assemble_address_record_relaxed: street-only (element_id={}), building partial record",
                        element_id
                    );
                    let record = AddressRecordBuilder::default()
                        .street(Some(street_only))
                        .build()
                        .map_err(|builder_err| {
                            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                                IncompatibleOsmPbfNode::AddressRecordBuilderError {
                                    id: element_id,
                                    source: builder_err,
                                },
                            )
                        })?;
                    Ok(Some(record))
                }
                (None, None, Some(zip_only)) => {
                    // Zip only => error
                    error!(
                        "try_assemble_address_record_relaxed: zip-only => returning error (element_id={})",
                        element_id
                    );
                    Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                        IncompatibleOsmPbfNode::PostalCodeConstructionError(
                            PostalCodeConstructionError::InvalidFormat {
                                attempted_code: zip_only.code().to_string(),
                                attempted_country: Some(*zip_only.country()),
                            }
                        ),
                    ))
                }
                _ => {
                    // unreachable if the logic above is correct
                    error!(
                        "try_assemble_address_record_relaxed: unexpected single-field scenario (element_id={})",
                        element_id
                    );
                    Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                        IncompatibleOsmPbfNode::Incompatible{ id: element_id }
                    ))
                }
            }
        }
        0 => {
            // No address fields => treat as “Incompatible / skip”
            error!(
                "try_assemble_address_record_relaxed: no city/street/postcode at all => error (element_id={})",
                element_id
            );
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible{ id: element_id }
            ))
        }
        _ => {
            // Should never happen, but we’ll handle gracefully
            error!(
                "try_assemble_address_record_relaxed: unexpected present_count={} for element_id={}",
                present_count, element_id
            );
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible{ id: element_id }
            ))
        }
    }
}

#[cfg(test)]
mod test_try_assemble_address_record {
    use super::*;
    use crate::errors::*;
    use derive_builder::UninitializedFieldError;

    /// Helpers for building valid city/street/postal
    fn valid_city() -> CityName {
        CityName::new("TestCity").unwrap()
    }
    fn valid_street() -> StreetName {
        StreetName::new("TestStreet").unwrap()
    }
    fn valid_postcode() -> PostalCode {
        PostalCode::new(Country::USA, "12345").unwrap()
    }

    // --------------------------------------------------------------------
    // (A) SCENARIO: all three fields => success
    // --------------------------------------------------------------------
    #[traced_test]
    fn test_all_three_fields_ok() {
        let city = Some(valid_city());
        let street = Some(valid_street());
        let postcode = Some(valid_postcode());
        let element_id = 100;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_ok(), "All three => should succeed");
        let record = result.unwrap();

        // Check that we got a record with all three fields
        assert!(record.is_some(), "Should return Some(record)");
        let rec = record.unwrap();
        assert_eq!(rec.city().as_ref().unwrap().name(), "testcity");
        assert_eq!(rec.street().as_ref().unwrap().name(), "teststreet");
        assert_eq!(rec.postcode().as_ref().unwrap().code(), "12345");
    }

    // --------------------------------------------------------------------
    // (B) SCENARIO: exactly two fields => success (no error)
    // --------------------------------------------------------------------
    #[traced_test]
    fn test_city_and_street_missing_zip_ok() {
        // city + street is valid (2 fields); we expect Ok( partial record ) + a possible warn
        let city = Some(valid_city());
        let street = Some(valid_street());
        let postcode = None;
        let element_id = 101;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_ok(), "City + Street => should succeed (partial).");
        let record = result.unwrap().unwrap();
        assert!(record.city().is_some());
        assert!(record.street().is_some());
        assert!(record.postcode().is_none());
    }

    #[traced_test]
    fn test_city_and_zip_missing_street_ok() {
        // city + zip => partial success
        let city = Some(valid_city());
        let postcode = Some(valid_postcode());
        let street = None;
        let element_id = 102;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_ok(), "City + Zip => partial success");
        let record = result.unwrap().unwrap();
        assert!(record.city().is_some());
        assert!(record.street().is_none());
        assert!(record.postcode().is_some());
    }

    #[traced_test]
    fn test_street_and_zip_missing_city_ok() {
        let city = None;
        let street = Some(valid_street());
        let postcode = Some(valid_postcode());
        let element_id = 103;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_ok(), "Street + Zip => partial success");
        let record = result.unwrap().unwrap();
        assert!(record.city().is_none());
        assert!(record.street().is_some());
        assert!(record.postcode().is_some());
    }

    // --------------------------------------------------------------------
    // (C) SCENARIO: exactly one field
    // --------------------------------------------------------------------
    #[traced_test]
    fn test_city_only_warns_but_ok() {
        // city-only => warns but returns Ok(Some(...))
        let city = Some(valid_city());
        let street = None;
        let postcode = None;
        let element_id = 200;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_ok(), "City-only => partial success with a warning");
        let record = result.unwrap().unwrap();
        assert!(record.city().is_some());
        assert!(record.street().is_none());
        assert!(record.postcode().is_none());
    }

    #[traced_test]
    fn test_street_only_warns_but_ok() {
        // street-only => warns but returns Ok(Some(...))
        let city = None;
        let street = Some(valid_street());
        let postcode = None;
        let element_id = 201;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_ok(), "Street-only => partial success with a warning");
        let record = result.unwrap().unwrap();
        assert!(record.city().is_none());
        assert!(record.street().is_some());
        assert!(record.postcode().is_none());
    }

    #[traced_test]
    fn test_zip_only_fails() {
        // zip-only => error
        let city = None;
        let street = None;
        let postcode = Some(valid_postcode());
        let element_id = 202;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_err(), "Zip-only => expected an error");
        // Check the actual error variant if you want:
        match result.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::PostalCodeConstructionError(_) => {
                        // Good enough check
                    }
                    other => panic!("Expected PostalCodeConstructionError for zip-only, got {:?}", other),
                }
            }
            other => panic!("Expected IncompatibleOsmPbfNode(...), got {:?}", other),
        }
    }

    // --------------------------------------------------------------------
    // (D) SCENARIO: no fields at all => error
    // --------------------------------------------------------------------
    #[traced_test]
    fn test_no_fields_fails() {
        let city = None;
        let street = None;
        let postcode = None;
        let element_id = 999;

        let result = try_assemble_address_record(city, street, postcode, element_id);
        assert!(result.is_err(), "No fields => must fail");
        match result.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id }
            ) => {
                assert_eq!(id, element_id, "Should store the node id in the error");
            }
            other => panic!("Expected IncompatibleOsmPbfNode::Incompatible, got {:?}", other),
        }
    }
}
