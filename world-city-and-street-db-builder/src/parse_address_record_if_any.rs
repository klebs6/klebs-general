// ---------------- [ File: src/parse_address_record_if_any.rs ]
crate::ix!();

/// Parses an [`AddressRecord`] from the element if possible, returning `Some(AddressRecord)`
/// or `None` if the element doesn't contain valid city/street/postcode tags.
pub fn parse_address_record_if_any(
    element: &osmpbf::Element,
    country: &Country
) -> Option<AddressRecord> {
    match AddressRecord::try_from((element, country)) {
        Ok(rec) => {
            debug!(
                "parse_address_record_if_any: successfully built an AddressRecord, city={:?}, street={:?}, postcode={:?}",
                rec.city(),
                rec.street(),
                rec.postcode()
            );
            Some(rec)
        }
        Err(e) => {
            debug!("parse_address_record_if_any: element not a valid address => {:?}", e);
            None
        }
    }
}

#[cfg(test)]
#[disable]
mod test_parse_address_record_if_any {
    use super::*;
    use osmpbf::{Element, Node};
    use std::collections::HashMap;

    /// A small helper to construct a mock `Node` with the given `id`
    /// and a set of `(key, value)` tags.  
    /// In real code, building an `osmpbf::Element` can be more involved,
    /// but for these tests, we can rely on the in-memory approach:
    fn make_node_with_tags(id: i64, tags: &[(&str, &str)]) -> Element<'static> {
        // Build an in-memory `Node` that implements `.tags()`.
        // We'll create a custom Node that holds these tag pairs in a `HashMap`.
        // Then wrap it as `Element::Node(...)`.
        let mut node = Node::default();
        node.set_id(id);

        // We'll store the tags in the node's internal data. The library's real usage may differ,
        // but for testing, we just want the `.tags()` method to yield our pairs.
        for (k, v) in tags {
            node.tags_mut().insert(k.to_string(), v.to_string());
        }

        Element::Node(node)
    }

    /// A small helper to confirm whether the returned `AddressRecord` matches
    /// our expected city/street/postcode, if any.
    fn assert_address_record_matches(
        actual: &AddressRecord,
        expected_city: Option<&str>,
        expected_street: Option<&str>,
        expected_postcode: Option<&str>,
    ) {
        match (expected_city, actual.city()) {
            (Some(exp_city), Some(actual_city)) => {
                assert_eq!(actual_city.name(), exp_city.to_lowercase());
            }
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => {
                panic!(
                    "Mismatch in city presence. Expected: {:?}, got: {:?}",
                    expected_city, actual.city()
                );
            }
        }

        match (expected_street, actual.street()) {
            (Some(exp_street), Some(actual_street)) => {
                assert_eq!(actual_street.name(), exp_street.to_lowercase());
            }
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => {
                panic!(
                    "Mismatch in street presence. Expected: {:?}, got: {:?}",
                    expected_street, actual.street()
                );
            }
        }

        match (expected_postcode, actual.postcode()) {
            (Some(exp_postcode), Some(actual_code)) => {
                assert_eq!(actual_code.code(), exp_postcode);
            }
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => {
                panic!(
                    "Mismatch in postcode presence. Expected: {:?}, got: {:?}",
                    expected_postcode, actual.postcode()
                );
            }
        }
    }

    /// We'll use `Country::USA` for all tests. Adjust as needed for your codebase.
    fn test_country() -> Country {
        Country::USA
    }

    #[test]
    fn test_valid_city_street_postcode_returns_some() {
        // Node with full set of valid tags => should parse.
        // e.g.: addr:city=Baltimore, addr:street=Main St, addr:postcode=21201
        let tags = &[
            ("addr:city", "Baltimore"),
            ("addr:street", "Main St"),
            ("addr:postcode", "21201"),
        ];
        let element = make_node_with_tags(1001, tags);

        let record_opt = parse_address_record_if_any(&element, &test_country());
        assert!(
            record_opt.is_some(),
            "Expected Some(AddressRecord) for valid city/street/postcode tags"
        );
        let record = record_opt.unwrap();
        assert_address_record_matches(
            &record,
            Some("Baltimore"),
            Some("Main St"),
            Some("21201"),
        );
    }

    #[test]
    fn test_no_addr_tags_returns_none() {
        // Node without any addr:city/addr:street/addr:postcode => None
        let tags = &[("highway", "residential"), ("name", "Random Road")];
        let element = make_node_with_tags(2002, tags);

        let record_opt = parse_address_record_if_any(&element, &test_country());
        assert!(
            record_opt.is_none(),
            "Expected None if no address tags are found"
        );
    }

    #[test]
    fn test_partial_addr_tags_returns_none() {
        // Node with only addr:city but missing street/postcode => should fail => None
        let tags = &[("addr:city", "Seattle")];
        let element = make_node_with_tags(3003, tags);

        let record_opt = parse_address_record_if_any(&element, &test_country());
        assert!(
            record_opt.is_none(),
            "Expected None if missing at least street or postcode"
        );
    }

    #[test]
    fn test_invalid_city_name_construction_returns_none() {
        // Suppose "CityName::new" or "AddressRecord::try_from" 
        // fails for an empty or invalid city name, leading to an error => None.
        // We'll simulate that with a city name that we expect to fail your constructor's validation.
        let tags = &[
            ("addr:city", ""), // or some invalid name
            ("addr:street", "Main St"),
            ("addr:postcode", "99999"),
        ];
        let element = make_node_with_tags(4004, tags);

        let record_opt = parse_address_record_if_any(&element, &test_country());
        assert!(
            record_opt.is_none(),
            "Expected None if city name fails validation"
        );
    }

    #[test]
    fn test_invalid_postal_code_for_country_returns_none() {
        // Suppose your `PostalCode::new` fails for some invalid format or mismatch with the country.
        // We'll assume "ABCDE1234" is invalid for the US in your code, for instance.
        let tags = &[
            ("addr:city", "Testville"),
            ("addr:street", "Test Street"),
            ("addr:postcode", "ABCDE1234"), 
        ];
        let element = make_node_with_tags(5005, tags);

        let record_opt = parse_address_record_if_any(&element, &test_country());
        assert!(
            record_opt.is_none(),
            "Expected None if postal code fails validation for the country"
        );
    }

    #[test]
    fn test_debug_logging_on_error() {
        // If parsing fails, the function logs a debug message and returns None.
        // We'll just confirm it returns None and rely on the logger to do its job
        // (no direct log checking, unless you have a log capture tool).
        let tags = &[
            ("addr:city", "ValidCity"),
            // Missing street => error
            ("addr:postcode", "21000"),
        ];
        let element = make_node_with_tags(6006, tags);

        let record_opt = parse_address_record_if_any(&element, &test_country());
        assert!(record_opt.is_none());
    }
}
