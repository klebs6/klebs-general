// ---------------- [ File: src/try_extract_address_tags.rs ]
// ---------------- [ File: src/try_extract_address_tags.rs ]
crate::ix!();

/// Searches a tag map for `addr:city`, `addr:street`, and `addr:postcode`.
/// Returns an error if none are present.
///
/// # Returns
///
/// * `Ok((city_raw, street_raw, postcode_raw))` if at least one is present.
/// * `Err(IncompatibleOsmPbfElement)` if all are absent.
pub fn try_extract_address_tags(
    tags: &std::collections::HashMap<String, String>,
    element_id: i64,
) -> Result<(Option<&str>, Option<&str>, Option<&str>), IncompatibleOsmPbfElement> {
    trace!("try_extract_address_tags: Looking for address tags in element_id={}", element_id);

    let city_raw     = tags.get("addr:city").map(|s| s.as_str());
    let street_raw   = tags.get("addr:street").map(|s| s.as_str());
    let postcode_raw = tags.get("addr:postcode").map(|s| s.as_str());

    if city_raw.is_none() && street_raw.is_none() && postcode_raw.is_none() {
        warn!("try_extract_address_tags: No addr:city/addr:street/addr:postcode for element_id={}", element_id);
        return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::Incompatible { id: element_id }
        ));
    }

    Ok((city_raw, street_raw, postcode_raw))
}

#[cfg(test)]
#[disable]
mod test_try_extract_address_tags {
    use super::*;
    use std::collections::HashMap;
    use tracing::{trace, warn};

    /// A small helper that builds a `HashMap<String, String>` from a slice of `(key, value)` tuples.
    fn build_tags(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), v.to_string());
        }
        map
    }

    #[traced_test]
    fn test_no_address_tags_returns_error() {
        // If city, street, and postcode are all missing => IncompatibleOsmPbfElement::IncompatibleOsmPbfNode => ...
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
        // The doc says returns an error if *none* are present; 
        // if city is present, that is "at least one," so it should be Ok((Some(...), None, None)).
        let tags = build_tags(&[("addr:city", "TestCity")]);
        let element_id = 1002;

        let result = try_extract_address_tags(&tags, element_id);
        assert!(result.is_ok(), "City only => success (some partial presence is allowed)");
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
