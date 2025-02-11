// ---------------- [ File: src/parse_integer.rs ]
// ---------------- [ File: src/parse_integer.rs ]
crate::ix!();

/// Parses a string as an unsigned integer (`u32`). Returns a domain error if invalid.
pub fn parse_integer(
    s: &str,
    element_id: i64,
) -> Result<u32, IncompatibleOsmPbfElement> {
    trace!(
        "parse_integer: parsing '{}' as u32 (element_id={})",
        s,
        element_id
    );

    s.parse::<u32>().map_err(|parse_err| {
        error!(
            "parse_integer: unable to parse '{}' as u32 (element_id={}): {}",
            s, element_id, parse_err
        );
        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::Incompatible { id: element_id }
        )
    })
}

#[cfg(test)]
mod test_parse_integer {
    use super::*;

    fn assert_ok_eq(
        result: Result<u32, IncompatibleOsmPbfElement>, 
        expected: u32
    ) {
        match result {
            Ok(val) => assert_eq!(val, expected, "Expected {}, got {}", expected, val),
            Err(e) => panic!("Expected Ok({}), got Err({:?})", expected, e),
        }
    }

    fn assert_err_incompatible_node(
        result: Result<u32, IncompatibleOsmPbfElement>, 
        expected_id: i64
    ) {
        match result {
            Ok(val) => panic!("Expected an error, but got Ok({})", val),
            Err(e) => match e {
                IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                    IncompatibleOsmPbfNode::Incompatible { id }
                ) => {
                    assert_eq!(
                        id, expected_id,
                        "Expected the error's element_id to be {}",
                        expected_id
                    );
                }
                other => panic!("Expected IncompatibleOsmPbfNode::Incompatible, got {:?}", other),
            }
        }
    }

    #[traced_test]
    fn test_valid_number() {
        // "123" => Ok(123)
        let element_id = 1001;
        let result = parse_integer("123", element_id);
        assert_ok_eq(result, 123);
    }

    #[traced_test]
    fn test_leading_whitespace() {
        // "   42" => Ok(42)
        let element_id = 1002;
        let result = parse_integer("   42", element_id);
        assert_ok_eq(result, 42);
    }

    #[traced_test]
    fn test_zero() {
        // "0" => Ok(0)
        let element_id = 1003;
        let result = parse_integer("0", element_id);
        assert_ok_eq(result, 0);
    }

    #[traced_test]
    fn test_negative_number() {
        // "-5" => Err(...)
        let element_id = 1004;
        let result = parse_integer("-5", element_id);
        assert_err_incompatible_node(result, element_id);
    }

    #[traced_test]
    fn test_non_numeric() {
        // "abc" => Err(...)
        let element_id = 1005;
        let result = parse_integer("abc", element_id);
        assert_err_incompatible_node(result, element_id);
    }

    #[traced_test]
    fn test_empty_string() {
        // "" => Err(...)
        let element_id = 1006;
        let result = parse_integer("", element_id);
        assert_err_incompatible_node(result, element_id);
    }

    #[traced_test]
    fn test_whitespace_only() {
        // "   " => Err(...)
        let element_id = 1007;
        let result = parse_integer("   ", element_id);
        assert_err_incompatible_node(result, element_id);
    }

    #[traced_test]
    fn test_u32_max() {
        // "4294967295" => Ok(4294967295) if it fits in u32
        let element_id = 1008;
        let result = parse_integer("4294967295", element_id);
        // 4294967295 = 2^32 - 1
        // This is exactly the maximum u32, so it should parse successfully.
        assert_ok_eq(result, std::u32::MAX);
    }

    #[traced_test]
    fn test_overflow_beyond_u32_max() {
        // "4294967296" => This is 2^32, out of range => Err(...)
        let element_id = 1009;
        let result = parse_integer("4294967296", element_id);
        assert_err_incompatible_node(result, element_id);
    }
}
