// ---------------- [ File: src/extract_house_number_range_from_tags.rs ]
crate::ix!();

/// Attempts to parse a house number or house‐number range from typical OSM tags:
///   - `addr:housenumber = "123"`        => returns Range(123..=123)
///   - `addr:housenumber = "100-150"`    => returns Range(100..=150)
///   - If none is found or unparseable, returns `Ok(None)`.
///
/// # Arguments
///
/// * `tags_iter`  - An iterator over (key, value) tag pairs.
/// * `element_id` - The ID of the OSM element from which the tags are drawn.
///
/// # Returns
///
/// * `Ok(Some(HouseNumberRange))` if a valid range was found.
/// * `Ok(None)` if no parseable house number is present.
/// * `Err(IncompatibleOsmPbfElement)` if an error prevents us from parsing.
pub fn extract_house_number_range_from_tags<'a, I>(
    tags_iter: I,
    element_id: i64,
) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement>
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    trace!(
        "extract_house_number_range_from_tags: start (element_id={})",
        element_id
    );

    let tags = collect_tags(tags_iter);
    debug!(
        "extract_house_number_range_from_tags: collected {} tags (element_id={})",
        tags.len(),
        element_id
    );

    match retrieve_housenumber_value(&tags, element_id)? {
        None => {
            debug!(
                "extract_house_number_range_from_tags: no housenumber tag found (element_id={})",
                element_id
            );
            Ok(None)
        }
        Some(raw_value) => parse_housenumber_value(raw_value, element_id),
    }
}

#[cfg(test)]
mod extract_house_number_range_from_tags_tests {
    use super::*;

    /// Helper to produce an iterator of (&str, &str).
    fn tag_iter<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Iterator<Item = (&'a str, &'a str)> {
        pairs.iter().map(|(k,v)| (*k,*v))
    }

    #[traced_test]
    fn test_no_housenumber_tag() {
        let pairs = [("addr:city", "Baltimore"), ("some_tag", "value")];
        let res = extract_house_number_range_from_tags(tag_iter(&pairs), 123);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
        // logs
        // assert!(logs_contain("no housenumber tag found (element_id=123)"));
    }

    #[traced_test]
    fn test_housenumber_empty() {
        let pairs = [("addr:housenumber", "   ")];
        let res = extract_house_number_range_from_tags(tag_iter(&pairs), 999);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_none());
        // assert!(logs_contain("no housenumber tag found (element_id=999)"));
    }

    #[traced_test]
    fn test_valid_single_number() {
        let pairs = [("addr:housenumber", "123")];
        let res = extract_house_number_range_from_tags(tag_iter(&pairs), 11);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_some());
        let rng = opt.unwrap();
        assert_eq!(*rng.start(), 123);
        assert_eq!(*rng.end(), 123);
        // assert!(logs_contain("collected 1 tags (element_id=11)"));
    }

    #[traced_test]
    fn test_valid_range() {
        let pairs = [("addr:housenumber", "100-150")];
        let result = extract_house_number_range_from_tags(tag_iter(&pairs), 22);
        assert!(result.is_ok());
        let opt = result.unwrap();
        let rng = opt.unwrap();
        assert_eq!((*rng.start(), *rng.end()), (100, 150));
    }

    #[traced_test]
    fn test_invalid_format_error() {
        // e.g. "ABC-XYZ" => parse error => IncompatibleOsmPbfNode::Incompatible{id=...}
        let pairs = [("addr:housenumber", "ABC-XYZ")];
        let result = extract_house_number_range_from_tags(tag_iter(&pairs), 33);
        assert!(result.is_err());
        match result.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(IncompatibleOsmPbfNode::Incompatible { id }) => {
                assert_eq!(id, 33);
            }
            other => panic!("Expected IncompatibleOsmPbfNode::Incompatible, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_additional_tags_ignored() {
        let pairs = [
            ("addr:city", "SomeCity"),
            ("any_tag", "any_val"),
            ("addr:housenumber", "999-1000"),
        ];
        let res = extract_house_number_range_from_tags(tag_iter(&pairs), 55);
        let rng_opt = res.unwrap();
        let rng = rng_opt.unwrap();
        assert_eq!((*rng.start(), *rng.end()), (999, 1000));
    }

    #[traced_test]
    fn test_large_number_within_u32() {
        // e.g. "4294967295" => that's (2^32)-1 => parse as u32 => Should either be an error
        // or if your code supports up to max u32, it is valid. We'll assume it's valid if your code uses u32.
        // If your code doesn't handle this well => might see an error
        // We'll see if we do an error or not:
        let pairs = [("addr:housenumber", "4294967295")];
        let res = extract_house_number_range_from_tags(tag_iter(&pairs), 66);
        // If the code is strictly "u32", 4294967295 => 0xFFFFFFFF => which is valid. 
        // If your parse returns an error, you'd adapt.
        match res {
            Ok(Some(rng)) => {
                assert_eq!(*rng.start(), 4294967295u32);
            }
            Err(e) => {
                panic!("We expected to handle max u32, got error: {:?}", e);
            }
            Ok(None) => {
                panic!("We expected Some(...), not None");
            }
        }
    }

    #[traced_test]
    fn test_range_reversed_ignored() {
        // e.g. "200-100" => since code checks if start_num > end_num => Ok(None)
        let pairs = [("addr:housenumber", "200-100")];
        let res = extract_house_number_range_from_tags(tag_iter(&pairs), 77);
        assert!(res.is_ok());
        let rng_opt = res.unwrap();
        // As the doc says, "if start_num > end_num => Ok(None)" or skip
        assert!(rng_opt.is_none(), "We expect reversed to yield None");
    }
}
