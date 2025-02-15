// ---------------- [ File: src/parse_housenumber_value.rs ]
crate::ix!();

/// Parses a non-empty housenumber string as either a single number or a range.
///
/// # Returns
///
/// * `Ok(Some(HouseNumberRange))` on success.
/// * `Ok(None)` if the start-end was reversed or invalid in an ignorable way.
/// * `Err(IncompatibleOsmPbfElement)` if a parse error occurs.
pub fn parse_housenumber_value(
    hn_value: &str,
    element_id: i64,
) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> {
    // Trim the incoming string
    let s = hn_value.trim();
    if s.is_empty() {
        // In this implementation, we interpret an empty or all-whitespace value
        // as "nothing to parse," hence Ok(None). You can decide whether
        // this should be an error instead, depending on your needs.
        debug!(
            "parse_housenumber_value: skipping empty string => no digits (element_id={})",
            element_id
        );
        return Ok(None);
    }

    trace!(
        "parse_housenumber_value: attempting to parse='{}' (element_id={})",
        s,
        element_id
    );

    // Split on '-' and trim around each portion
    let parts: Vec<&str> = s.split('-').map(|part| part.trim()).collect();

    match parts.len() {
        1 => {
            // No dash present => parse as a single integer
            let number = parse_integer(parts[0], element_id)?;
            let range = HouseNumberRange::new(number, number);
            debug!(
                "parse_housenumber_value: parsed single '{}' => {:?} (element_id={})",
                s, range, element_id
            );
            Ok(Some(range))
        }
        2 => {
            // One dash present => parse start and end
            let start_num = parse_integer(parts[0], element_id)?;
            let end_num = parse_integer(parts[1], element_id)?;

            if start_num > end_num {
                // We treat reversed or invalid ranges as ignorable => Ok(None)
                debug!(
                    "parse_housenumber_value: reversed or invalid range '{}-{}' => ignoring (element_id={})",
                    start_num, end_num, element_id
                );
                Ok(None)
            } else {
                let range = HouseNumberRange::new(start_num, end_num);
                debug!(
                    "parse_housenumber_value: parsed valid range '{}' => {:?} (element_id={})",
                    s, range, element_id
                );
                Ok(Some(range))
            }
        }
        _ => {
            // More than one dash => cannot parse it properly
            debug!(
                "parse_housenumber_value: invalid format => multiple dashes => parse error (element_id={})",
                element_id
            );
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id: element_id },
            ))
        }
    }
}

#[cfg(test)]
mod test_parse_housenumber_value {
    use super::*;

    /// Convenience for printing more descriptive messages when a result is expected to be `Ok(Some(...))`.
    fn assert_ok_some(
        result: Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement>,
        expected_start: u32,
        expected_end: u32,
    ) {
        match result {
            Ok(Some(range)) => {
                assert_eq!(*range.start(), expected_start, "Range start mismatch");
                assert_eq!(*range.end(), expected_end, "Range end mismatch");
            }
            Ok(None) => panic!("Expected Some(HouseNumberRange), got Ok(None)"),
            Err(e) => panic!("Expected Some(HouseNumberRange), got Err({:?})", e),
        }
    }

    /// Convenience for asserting `Ok(None)`.
    fn assert_ok_none(
        result: Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement>,
    ) {
        match result {
            Ok(Some(r)) => panic!("Expected Ok(None) but got Ok(Some({:?}))", r),
            Ok(None) => { /* as expected */ }
            Err(e) => panic!("Expected Ok(None), got Err({:?})", e),
        }
    }

    /// Convenience for asserting we got an error (i.e., `Err(...)`).
    fn assert_err(
        result: Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement>,
    ) -> IncompatibleOsmPbfElement {
        match result {
            Ok(r) => panic!("Expected an error, got Ok({:?})", r),
            Err(e) => e,
        }
    }

    #[traced_test]
    fn test_parse_single_integer() {
        // "123" => HouseNumberRange(123..=123)
        let res = parse_housenumber_value("123", 1);
        assert_ok_some(res, 123, 123);
    }

    #[traced_test]
    fn test_parse_simple_range() {
        // "10-20" => HouseNumberRange(10..=20)
        let res = parse_housenumber_value("10-20", 100);
        assert_ok_some(res, 10, 20);
    }

    #[traced_test]
    fn test_parse_range_with_whitespace() {
        // "  10  -  20  " => HouseNumberRange(10..=20)
        let res = parse_housenumber_value("  10  -  20  ", 999);
        assert_ok_some(res, 10, 20);
    }

    #[traced_test]
    fn test_reversed_range_returns_ok_none() {
        // "30-20" => reversed => Ok(None)
        let res = parse_housenumber_value("30-20", 2);
        assert_ok_none(res);
    }

    #[traced_test]
    fn test_parse_error_non_numeric() {
        // e.g. "ABC" => parse error => Err(IncompatibleOsmPbfElement)
        let res = parse_housenumber_value("ABC", 3);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => {
                        assert_eq!(id, 3);
                    }
                    _ => panic!(
                        "Expected Incompatible node error variant, got {:?}",
                        node_err
                    ),
                }
            }
            other => panic!("Expected IncompatibleOsmPbfNode, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_parse_error_non_numeric_range_end() {
        // "10-XYZ" => parse error => Err(...)
        let res = parse_housenumber_value("10-XYZ", 4);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => {
                        assert_eq!(id, 4);
                    }
                    _ => panic!(
                        "Expected Incompatible node error variant, got {:?}",
                        node_err
                    ),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode variant"),
        }
    }

    #[traced_test]
    fn test_parse_error_negative_number() {
        // negative number is not a valid u32 => parse error => Err(...)
        let res = parse_housenumber_value("-5", 5);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => {
                        assert_eq!(id, 5);
                    }
                    _ => panic!(
                        "Expected Incompatible node error variant, got {:?}",
                        node_err
                    ),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode variant"),
        }
    }

    #[traced_test]
    fn test_range_with_extra_dash_in_end_portion() {
        // "10-20-30" => we do split on '-', giving ["10", "20", "30"], parse => fail
        let res = parse_housenumber_value("10-20-30", 6);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => {
                        assert_eq!(id, 6);
                    }
                    _ => panic!(
                        "Expected Incompatible node error variant, got {:?}",
                        node_err
                    ),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode variant"),
        }
    }

    #[traced_test]
    fn test_zero_as_valid_single_number() {
        // "0" => HouseNumberRange(0..=0)
        let res = parse_housenumber_value("0", 10);
        assert_ok_some(res, 0, 0);
    }

    #[traced_test]
    fn test_leading_dash_is_non_numeric() {
        // "- 20" => split => ["", "20"] => the first piece is empty => parse fails
        let res = parse_housenumber_value("- 20", 11);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => {
                        assert_eq!(id, 11);
                    }
                    _ => panic!(
                        "Expected Incompatible node error variant, got {:?}",
                        node_err
                    ),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode variant"),
        }
    }
}
