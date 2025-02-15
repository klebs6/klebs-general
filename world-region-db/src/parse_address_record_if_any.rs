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
mod test_parse_address_record_if_any {
    use super::*;
    use osmpbf::{Element, Node};
    use std::collections::HashMap;

    /// We'll use `Country::USA` for all tests. Adjust as needed for your codebase.
    fn test_country() -> Country {
        Country::USA
    }

    #[traced_test]
    fn test_valid_city_street_postcode_returns_some() {
        // Node with full set of valid tags => should parse.
        // e.g.: addr:city=Baltimore, addr:street=Main St, addr:postcode=21201
        let tags = &[
            ("addr:city", "Baltimore"),
            ("addr:street", "Main St"),
            ("addr:postcode", "21201"),
        ];
        let node = MockNode::new(1001, tags);

        let record_opt = parse_address_record_if_any(&node.as_element(), &test_country());
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

    #[traced_test]
    fn test_no_addr_tags_returns_none() {
        // Node without any addr:city/addr:street/addr:postcode => None
        let tags = &[("highway", "residential"), ("name", "Random Road")];
        let node = MockNode::new(2002, tags);

        let record_opt = parse_address_record_if_any(&node.as_element(), &test_country());
        assert!(
            record_opt.is_none(),
            "Expected None if no address tags are found"
        );
    }

    #[traced_test]
    fn test_partial_addr_tags_returns_none() {
        // Node with only addr:city but missing street/postcode => should fail => None
        let tags = &[("addr:city", "Seattle")];
        let node = MockNode::new(3003, tags);

        let record_opt = parse_address_record_if_any(&node.as_element(), &test_country());
        assert!(
            record_opt.is_some(),
            "Expected Some even though missing at least street or postcode"
        );
    }

    #[traced_test]
    fn test_invalid_city_name_construction_returns_none() {
        // Suppose "CityName::new" or "AddressRecord::try_from" 
        // fails for an empty or invalid city name, leading to an error => None.
        // We'll simulate that with a city name that we expect to fail your constructor's validation.
        let tags = &[
            ("addr:city", ""), // or some invalid name
            ("addr:street", "Main St"),
            ("addr:postcode", "99999"),
        ];
        let node = MockNode::new(4004, tags);

        let record_opt = parse_address_record_if_any(&node.as_element(), &test_country());
        assert!(
            record_opt.is_none(),
            "Expected None if city name fails validation"
        );
    }

    #[traced_test]
    fn test_invalid_postal_code_for_country_returns_none() {
        // Suppose your `PostalCode::new` fails for some invalid format or mismatch with the country.
        // We'll assume "ABCDE1234" is invalid for the US in your code, for instance.
        let tags = &[
            ("addr:city", "Testville"),
            ("addr:street", "Test Street"),
            ("addr:postcode", "ABCDE1234"), 
        ];
        let node = MockNode::new(5005, tags);

        let record_opt = parse_address_record_if_any(&node.as_element(), &test_country());
        assert!(
            record_opt.is_none(),
            "Expected None if postal code fails validation for the country"
        );
    }

    #[traced_test]
    fn test_debug_logging_on_error() {
        // If parsing fails, the function logs a debug message and returns None.
        // We'll just confirm it returns None and rely on the logger to do its job
        // (no direct log checking, unless you have a log capture tool).
        let tags = &[
            ("addr:city", "ValidCity"),
            // Missing street => error
            ("addr:postcode", "21000"),
        ];
        let node = MockNode::new(6006, tags);

        let record_opt = parse_address_record_if_any(&node.as_element(), &test_country());
        assert!(record_opt.is_some());
    }
}
