// ---------------- [ File: src/try_build_address_record_from_tags.rs ]
// ---------------- [ File: src/try_build_address_record_from_tags.rs ]
crate::ix!();

/// Attempts to construct an [`AddressRecord`] from a stream of OSM-style tags.
/// Returns an error if no address-related tags are found or if any field fails
/// to parse.
///
/// # Arguments
///
/// * `tags_iter`  - Iterator of key-value pairs representing OSM tags.
/// * `country`    - The country associated with the address record.
/// * `element_id` - Unique identifier (e.g., node id).
///
/// # Returns
///
/// * `Ok(AddressRecord)` if a valid record can be built.
/// * `Err(IncompatibleOsmPbfElement)` otherwise.
pub fn try_build_address_record_from_tags<'a>(
    tags_iter: impl Iterator<Item = (&'a str, &'a str)>,
    country: Country,
    element_id: i64,
) -> Result<AddressRecord, IncompatibleOsmPbfElement> {
    trace!("try_build_address_record_from_tags: Start for element_id={}", element_id);

    let tags = collect_tags(tags_iter);
    debug!(
        "try_build_address_record_from_tags: Collected {} tags for element_id={}",
        tags.len(),
        element_id
    );

    // 1. Extract city/street/postcode tags or return an error if all missing.
    let (city_raw, street_raw, postcode_raw) = try_extract_address_tags(&tags, element_id)?;

    // 2. Parse each tag into its strongly-typed representation.
    let city     = try_construct_city_name(city_raw, element_id)?;
    let street   = try_construct_street_name(street_raw, element_id)?;
    let postcode = try_construct_postal_code(country, postcode_raw, element_id)?;

    // 3. Assemble the final [`AddressRecord`].
    let record = try_assemble_address_record(city, street, postcode, element_id)?;

    info!("try_build_address_record_from_tags: Successfully built AddressRecord for element_id={}", element_id);
    Ok(record)
}

#[cfg(test)]
#[disable]
mod test_try_build_address_record_from_tags {
    use super::*;
    use crate::errors::*;
    use std::collections::HashMap;

    /// A helper that creates a mock iterator of (&str, &str) from a slice of (key, value) pairs.
    fn make_tag_iter(tags: &[(&str, &str)]) -> impl Iterator<Item = (&'_ str, &'_ str)> {
        tags.iter().map(|&(k, v)| (k, v))
    }

    /// Utility function for constructing an OSM-style key-value map from pairs.
    /// In actual usage, `collect_tags(...)` is used internally by the function,
    /// but we can replicate for testing or simply pass the iterator.
    fn build_tags_map(tags: &[(&str, &str)]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in tags {
            map.insert(k.to_string(), v.to_string());
        }
        map
    }

    fn test_country() -> Country {
        // For this test, we'll consistently use the USA
        Country::USA
    }

    /// A convenience for checking the success path.
    /// If successful, returns the `AddressRecord`.
    fn assert_ok_address(
        result: Result<AddressRecord, IncompatibleOsmPbfElement>,
        expected_city: &str,
        expected_street: &str,
        expected_postcode: &str
    ) -> AddressRecord {
        match result {
            Ok(rec) => {
                // Compare city/street/postcode ignoring case, 
                // or as your code does. We'll do lowercase for convenience.
                assert_eq!(rec.city().unwrap().name(), expected_city.to_lowercase());
                assert_eq!(rec.street().unwrap().name(), expected_street.to_lowercase());
                assert_eq!(rec.postcode().unwrap().code(), expected_postcode);
                rec
            }
            Err(e) => panic!("Expected Ok(AddressRecord), got Err({:?})", e),
        }
    }

    /// Convenience for checking the error path. Returns the error variant.
    fn assert_err(result: Result<AddressRecord, IncompatibleOsmPbfElement>) -> IncompatibleOsmPbfElement {
        match result {
            Ok(rec) => panic!("Expected an error, but got Ok({:?})", rec),
            Err(e) => e,
        }
    }

    #[traced_test]
    fn test_successful_build() {
        // Node has addr:city, addr:street, addr:postcode => success
        let tags = &[
            ("addr:city", "Baltimore"),
            ("addr:street", "North Avenue"),
            ("addr:postcode", "21201"),
            ("unrelated", "foo"),
        ];

        let element_id = 1001;
        let result = try_build_address_record_from_tags(
            make_tag_iter(tags),
            test_country(),
            element_id
        );

        // Expect city="Baltimore", street="North Avenue", postcode="21201"
        // We'll do a quick check ignoring case for city/street:
        assert_ok_address(result, "Baltimore", "North Avenue", "21201");
    }

    #[traced_test]
    fn test_no_address_tags_returns_error() {
        // If none of addr:city, addr:street, addr:postcode are present, we get an error
        let tags = &[("highway", "residential"), ("name", "just a random feature")];
        let element_id = 2002;

        let result = try_build_address_record_from_tags(
            make_tag_iter(tags),
            test_country(),
            element_id
        );
        let err = assert_err(result);

        // Should be an IncompatibleOsmPbfElement => IncompatibleOsmPbfNode(...) with the "Incompatible { id }" variant or similar.
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => {
                        assert_eq!(id, element_id, "Should match the element_id we used");
                    }
                    other => panic!("Expected Incompatible {{ id }} variant, got {:?}", other),
                }
            }
            other => panic!("Expected IncompatibleOsmPbfNode variant, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_missing_street_or_city_or_postcode_returns_error() {
        // The function requires all three to exist in the tags or it fails with Incompatible.
        
        // Only city => fail
        let tags_city_only = &[("addr:city", "SomeCity")];
        let result_city_only = try_build_address_record_from_tags(
            make_tag_iter(tags_city_only),
            test_country(),
            3003
        );
        assert_err(result_city_only);

        // Only street => fail
        let tags_street_only = &[("addr:street", "SomeStreet")];
        let result_street_only = try_build_address_record_from_tags(
            make_tag_iter(tags_street_only),
            test_country(),
            3004
        );
        assert_err(result_street_only);

        // Only postcode => fail
        let tags_postcode_only = &[("addr:postcode", "99999")];
        let result_postcode_only = try_build_address_record_from_tags(
            make_tag_iter(tags_postcode_only),
            test_country(),
            3005
        );
        assert_err(result_postcode_only);
    }

    #[traced_test]
    fn test_invalid_city_name_construction_error() {
        // Suppose an empty city => CityName::new("") fails => error
        let tags = &[
            ("addr:city", ""), // invalid city
            ("addr:street", "TestStreet"),
            ("addr:postcode", "11111"),
        ];

        let res = try_build_address_record_from_tags(make_tag_iter(tags), test_country(), 4004);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::CityNameConstructionError(_) => {
                        // That means city parse failed as expected
                    }
                    other => panic!("Expected CityNameConstructionError, got {:?}", other),
                }
            }
            other => panic!("Expected IncompatibleOsmPbfNode for city parse fail, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_invalid_street_name_construction_error() {
        // Similar scenario: empty or invalid street => fails
        let tags = &[
            ("addr:city", "City"),
            ("addr:street", ""), // invalid street
            ("addr:postcode", "22222"),
        ];

        let res = try_build_address_record_from_tags(make_tag_iter(tags), test_country(), 5005);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::StreetNameConstructionError(_) => {
                        // Good
                    }
                    _ => panic!("Expected StreetNameConstructionError, got {:?}", node_err),
                }
            }
            other => panic!("Expected IncompatibleOsmPbfNode, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_invalid_postcode_construction_error() {
        // If the postal code is invalid for the country => error
        // We'll simulate with something that might fail for the US
        let tags = &[
            ("addr:city", "City"),
            ("addr:street", "Street"),
            ("addr:postcode", "InvalidPC"),
        ];

        let res = try_build_address_record_from_tags(make_tag_iter(tags), test_country(), 6006);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::PostalCodeConstructionError(_) => {
                        // Good
                    }
                    _ => panic!("Expected PostalCodeConstructionError, got {:?}", node_err),
                }
            }
            other => panic!("Expected IncompatibleOsmPbfNode variant, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_builder_fails_on_addressrecord_build() {
        // Even if city/street/postcode parse is okay, the final `try_assemble_address_record` might fail
        // if the builder enforces something else. We'll simulate an address record builder 
        // that requires a non-empty city. Actually we tested partial, let's do a scenario 
        // where the builder logic fails in an odd way.

        // We'll pass valid city/street/postcode strings but forcibly cause the builder to fail 
        // by mocking or adjusting the builder. For demonstration, let's do city=some invalid name 
        // that might parse initially but fails the builder constraints. 
        // This is somewhat contrived, as the city parse would likely fail earlier. 
        // We'll proceed to show the approach.

        let tags = &[
            ("addr:city", "ImpostorCity"),
            ("addr:street", "Main St"),
            ("addr:postcode", "99999"),
        ];

        // If your real builder doesn't add extra constraints, this might pass. 
        // We'll just show a test expecting an error if you do have constraints.
        let res = try_build_address_record_from_tags(make_tag_iter(tags), test_country(), 7007);

        // If your builder passes it, replace the following with an assert_ok_address 
        // to confirm success.
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    // If the builder fails => AddressRecordBuilderError
                    IncompatibleOsmPbfNode::AddressRecordBuilderError { .. } => {
                        // Good
                    }
                    _ => panic!("Expected AddressRecordBuilderError, got {:?}", node_err),
                }
            }
            other => panic!("Expected IncompatibleOsmPbfNode, got {:?}", other),
        }
    }
}
