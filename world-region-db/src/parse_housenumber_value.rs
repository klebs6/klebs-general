// ---------------- [ File: src/parse_housenumber_value.rs ]
crate::ix!();

/// Parses a non-empty housenumber string as either a single number or a range,
/// allowing trailing non-digit text. For example, "2801 B" is parsed as 2801.
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
    let s = hn_value.trim();
    if s.is_empty() {
        // If all whitespace, there's nothing to parse.
        debug!(
            "parse_housenumber_value: skipping empty/whitespace for element_id={} => no digits",
            element_id
        );
        return Ok(None);
    }

    trace!(
        "parse_housenumber_value: attempting to parse='{}' (element_id={})",
        s,
        element_id
    );

    // Split into at most two parts. More than that => error.
    let parts: Vec<&str> = s.split('-').map(str::trim).collect();
    match parts.len() {
        1 => {
            // No dash => parse one integer prefix
            let number = parse_integer_prefix(parts[0], element_id)?;
            let range = HouseNumberRange::new(number, number);
            debug!(
                "parse_housenumber_value: parsed single '{}' => {:?} (element_id={})",
                s, range, element_id
            );
            Ok(Some(range))
        }
        2 => {
            // One dash => parse both sides
            let start_num = parse_integer_prefix(parts[0], element_id)?;
            let end_num = parse_integer_prefix(parts[1], element_id)?;

            if start_num > end_num {
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
            // More than one dash => error
            debug!(
                "parse_housenumber_value: multiple dashes => parse error (element_id={})",
                element_id
            );
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id: element_id },
            ))
        }
    }
}

/// Parses the **leading digit sequence** in the given string. Returns an error if
/// there are no leading digits or if the leading digits cannot fit into `u32`.
fn parse_integer_prefix(
    input: &str,
    element_id: i64,
) -> Result<u32, IncompatibleOsmPbfElement> {
    // Extract only the leading digits
    let digits: String = input.chars().take_while(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        debug!(
            "parse_integer_prefix: no digits in '{}' (element_id={}) => parse error",
            input, element_id
        );
        return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::Incompatible { id: element_id },
        ));
    }

    match digits.parse::<u32>() {
        Ok(val) => Ok(val),
        Err(e) => {
            debug!(
                "parse_integer_prefix: unable to parse '{}' as u32 (element_id={}): {:?}",
                digits, element_id, e
            );
            Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id: element_id },
            ))
        }
    }
}

#[cfg(test)]
mod test_parse_housenumber_value_with_prefix {
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
        let res = parse_housenumber_value("123", 1);
        assert_ok_some(res, 123, 123);
    }

    #[traced_test]
    fn test_parse_simple_range() {
        let res = parse_housenumber_value("10-20", 2);
        assert_ok_some(res, 10, 20);
    }

    #[traced_test]
    fn test_parse_range_with_whitespace() {
        let res = parse_housenumber_value("  10  -  20  ", 3);
        assert_ok_some(res, 10, 20);
    }

    #[traced_test]
    fn test_reversed_range_returns_ok_none() {
        let res = parse_housenumber_value("30-20", 4);
        assert_ok_none(res);
    }

    #[traced_test]
    fn test_non_numeric_is_error() {
        let res = parse_housenumber_value("ABC", 5);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => assert_eq!(id, 5),
                    _ => panic!("Expected Incompatible node error, got {:?}", node_err),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode, got {:?}", err),
        }
    }

    #[traced_test]
    fn test_parse_error_non_numeric_range_end() {
        let res = parse_housenumber_value("10-XYZ", 6);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => assert_eq!(id, 6),
                    _ => panic!("Expected Incompatible node error, got {:?}", node_err),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode variant"),
        }
    }

    #[traced_test]
    fn test_parse_error_negative_number() {
        let res = parse_housenumber_value("-5", 7);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => assert_eq!(id, 7),
                    _ => panic!("Expected Incompatible node error, got {:?}", node_err),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode variant"),
        }
    }

    #[traced_test]
    fn test_range_with_extra_dash_in_end_portion() {
        // multiple dashes => error
        let res = parse_housenumber_value("10-20-30", 8);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => assert_eq!(id, 8),
                    _ => panic!("Expected Incompatible node error, got {:?}", node_err),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode variant"),
        }
    }

    #[traced_test]
    fn test_zero_as_valid_single_number() {
        let res = parse_housenumber_value("0", 9);
        assert_ok_some(res, 0, 0);
    }

    #[traced_test]
    fn test_leading_dash_is_non_numeric() {
        // first half has no digits
        let res = parse_housenumber_value("- 20", 10);
        let err = assert_err(res);
        match err {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                match node_err {
                    IncompatibleOsmPbfNode::Incompatible { id } => assert_eq!(id, 10),
                    _ => panic!("Expected Incompatible node error, got {:?}", node_err),
                }
            }
            _ => panic!("Expected IncompatibleOsmPbfNode variant"),
        }
    }

    #[traced_test]
    fn test_trailing_letters_in_single_number() {
        // "2801 B" => parse leading digits => HouseNumberRange(2801..=2801)
        let res = parse_housenumber_value("2801 B", 11);
        assert_ok_some(res, 2801, 2801);
    }

    #[traced_test]
    fn test_trailing_letters_in_range() {
        // "100A-200B" => parse leading digits on both => HouseNumberRange(100..=200)
        let res = parse_housenumber_value("100A - 200B", 12);
        assert_ok_some(res, 100, 200);
    }
}
