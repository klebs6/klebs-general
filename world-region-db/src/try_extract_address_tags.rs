// ---------------- [ File: src/try_extract_address_tags.rs ]
crate::ix!();

/// Tries to extract address tags from the provided `tags` map.
/// If at least one of `addr:city`, `addr:street`, or `addr:postcode` is present,
/// returns a tuple of optional string slices corresponding to these tags.
/// Returns an error if none of these tags are present.
pub fn try_extract_address_tags(
    tags: &HashMap<String, String>,
    element_id: i64,
) -> Result<(Option<&str>, Option<&str>, Option<&str>), IncompatibleOsmPbfElement> {

    let city_opt     = tags.get("addr:city").map(|s| s.as_str());
    let street_opt   = tags.get("addr:street").map(|s| s.as_str());
    let postcode_opt = tags.get("addr:postcode").map(|s| s.as_str());

    // If none of the address tags are present, return an error.
    if city_opt.is_none() && street_opt.is_none() && postcode_opt.is_none() {
        Err(
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id: element_id }
            )
        )
    } else {
        Ok((city_opt, street_opt, postcode_opt))
    }
}

#[cfg(test)]
mod test_try_extract_address_tags {
    use super::*;
    use std::collections::HashMap;
    use tracing::{trace, warn};

    /// Helper that builds a `HashMap<String, String>` from a slice of `(key, value)` pairs.
    fn build_tags(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), v.to_string());
        }
        map
    }

    #[traced_test]
    fn test_no_address_tags_returns_error() {
        let tags = build_tags(&[("name", "JustAFeature"), ("highway", "residential")]);
        let element_id = 1001;

        let result = try_extract_address_tags(&tags, element_id);
        match result {
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id }
            )) => {
                assert_eq!(id, element_id, "Expected the same element id in the error");
            }
            other => panic!("Expected an Incompatible node error, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_only_city_tag_returns_ok() {
        let tags = build_tags(&[("addr:city", "TestCity")]);
        let element_id = 1002;

        let result = try_extract_address_tags(&tags, element_id);
        assert!(result.is_ok(), "City only => success (partial presence is allowed)");
        let (city_opt, street_opt, postcode_opt) = result.unwrap();
        assert_eq!(city_opt, Some("TestCity"));
        assert!(street_opt.is_none());
        assert!(postcode_opt.is_none());
    }

    #[traced_test]
    fn test_only_street_tag_returns_ok() {
        let tags = build_tags(&[("addr:street", "TestStreet")]);
        let element_id = 1003;

        let result = try_extract_address_tags(&tags, element_id);
        assert!(result.is_ok(), "Street only => success");
        let (city_opt, street_opt, postcode_opt) = result.unwrap();
        assert!(city_opt.is_none());
        assert_eq!(street_opt, Some("TestStreet"));
        assert!(postcode_opt.is_none());
    }

    #[traced_test]
    fn test_only_postcode_tag_returns_ok() {
        let tags = build_tags(&[("addr:postcode", "12345")]);
        let element_id = 1004;

        let result = try_extract_address_tags(&tags, element_id);
        assert!(result.is_ok(), "Postcode only => success");
        let (city_opt, street_opt, postcode_opt) = result.unwrap();
        assert!(city_opt.is_none());
        assert!(street_opt.is_none());
        assert_eq!(postcode_opt, Some("12345"));
    }

    #[traced_test]
    fn test_all_three_tags_returns_ok() {
        let tags = build_tags(&[
            ("addr:city", "CityVal"),
            ("addr:street", "StreetVal"),
            ("addr:postcode", "99999"),
            ("other", "whatever"),
        ]);
        let element_id = 1005;

        let result = try_extract_address_tags(&tags, element_id);
        assert!(result.is_ok(), "All present => definitely success");
        let (city_opt, street_opt, postcode_opt) = result.unwrap();
        assert_eq!(city_opt, Some("CityVal"));
        assert_eq!(street_opt, Some("StreetVal"));
        assert_eq!(postcode_opt, Some("99999"));
    }

    #[traced_test]
    fn test_partial_tags_city_and_postcode_returns_ok() {
        let tags = build_tags(&[
            ("addr:city", "PartialCity"),
            ("addr:postcode", "PartialPostcode"),
        ]);
        let element_id = 1006;

        let result = try_extract_address_tags(&tags, element_id);
        assert!(result.is_ok(), "At least one of city/street/postcode => success");
        let (city_opt, street_opt, postcode_opt) = result.unwrap();
        assert_eq!(city_opt, Some("PartialCity"));
        assert!(street_opt.is_none());
        assert_eq!(postcode_opt, Some("PartialPostcode"));
    }
}
