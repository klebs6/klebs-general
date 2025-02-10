// ---------------- [ File: src/retrieve_housenumber_value.rs ]
// ---------------- [ File: src/retrieve_housenumber_value.rs ]
crate::ix!();

/// Retrieves the `addr:housenumber` value from the collected tags, if present and non-empty.
///
/// # Returns
///
/// * `Ok(None)` if the housenumber key is absent or empty.
/// * `Ok(Some(&str))` containing a trimmed housenumber string otherwise.
/// * `Err(...)` if the data is invalid in a way that must produce an error.
pub fn retrieve_housenumber_value(
    tags: &HashMap<String, String>,
    element_id: i64,
) -> Result<Option<&str>, IncompatibleOsmPbfElement> {
    trace!(
        "retrieve_housenumber_value: checking for addr:housenumber (element_id={})",
        element_id
    );

    match tags.get("addr:housenumber") {
        None => Ok(None),
        Some(val) if val.trim().is_empty() => Ok(None),
        Some(val) => {
            debug!(
                "retrieve_housenumber_value: found housenumber='{}' (element_id={})",
                val, element_id
            );
            Ok(Some(val.trim()))
        }
    }
}

#[cfg(test)]
#[disable]
mod test_retrieve_housenumber_value {
    use super::*;
    use std::collections::HashMap;

    /// A helper to build a `HashMap<String, String>` from a list of (key, value) pairs.
    fn build_tags(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            map.insert(k.to_string(), v.to_string());
        }
        map
    }

    /// Convenience for asserting `Ok(Some(hn))` with a specific expected string.
    fn assert_ok_some(
        result: Result<Option<&str>, IncompatibleOsmPbfElement>,
        expected: &str,
    ) {
        match result {
            Ok(Some(hn)) => {
                assert_eq!(hn, expected, "Expected housenumber '{}', got '{}'", expected, hn);
            }
            Ok(None) => panic!("Expected Some(\"{}\"), got None", expected),
            Err(e) => panic!("Expected Some(\"{}\"), got Err({:?})", expected, e),
        }
    }

    /// Convenience for asserting `Ok(None)`.
    fn assert_ok_none(result: Result<Option<&str>, IncompatibleOsmPbfElement>) {
        match result {
            Ok(Some(val)) => panic!("Expected None, got Some(\"{}\")", val),
            Ok(None) => { /* as expected */ },
            Err(e) => panic!("Expected Ok(None), got Err({:?})", e),
        }
    }

    /// Convenience for asserting any error case (`Err(...)`).
    /// If your function never returns an error for invalid strings, you might not need this.
    fn assert_err(result: Result<Option<&str>, IncompatibleOsmPbfElement>) {
        match result {
            Ok(maybe_val) => {
                panic!("Expected an error, got Ok({:?})", maybe_val);
            }
            Err(_e) => { /* as expected */ }
        }
    }

    #[traced_test]
    fn test_no_housenumber_key_returns_ok_none() {
        // If "addr:housenumber" is not present, we get Ok(None).
        let tags = build_tags(&[("some_key", "some_val")]);
        let element_id = 1;
        let result = retrieve_housenumber_value(&tags, element_id);
        assert_ok_none(result);
    }

    #[traced_test]
    fn test_empty_housenumber_value_returns_ok_none() {
        // "addr:housenumber" => "" => trimmed is empty => Ok(None)
        let tags = build_tags(&[("addr:housenumber", "")]);
        let element_id = 2;
        let result = retrieve_housenumber_value(&tags, element_id);
        assert_ok_none(result);
    }

    #[traced_test]
    fn test_whitespace_housenumber_value_returns_ok_none() {
        // "addr:housenumber" => "   " => still empty after trim => Ok(None)
        let tags = build_tags(&[("addr:housenumber", "   ")]);
        let element_id = 3;
        let result = retrieve_housenumber_value(&tags, element_id);
        assert_ok_none(result);
    }

    #[traced_test]
    fn test_valid_housenumber_returns_ok_some_trimmed() {
        // "addr:housenumber" => "  123 " => trimmed => "123"
        let tags = build_tags(&[("addr:housenumber", "  123 ")]);
        let element_id = 4;
        let result = retrieve_housenumber_value(&tags, element_id);
        assert_ok_some(result, "123");
    }

    #[traced_test]
    fn test_another_valid_housenumber_with_dashes() {
        // This function doesn't parse the number, just retrieves it. 
        // "addr:housenumber" => "10-20"
        let tags = build_tags(&[("addr:housenumber", "10-20")]);
        let element_id = 5;
        let result = retrieve_housenumber_value(&tags, element_id);
        // It returns "10-20" trimmed. The actual range parse is done elsewhere.
        assert_ok_some(result, "10-20");
    }

    // The docstring mentions "*Err(...) if the data is invalid in a way that must produce an error*".
    // Currently, the function does not produce an error for any invalid input. 
    // If you add logic for certain error conditions, you'd add corresponding tests here.
    // For demonstration, let's simulate an extension: if the function decided 
    // to handle some special invalid case. We'll just show how we'd test it:

    #[traced_test]
    #[ignore = "Currently retrieve_housenumber_value never returns Err(...) for invalid data. 
                Un-ignore if you add that feature in the future."]
    fn test_special_invalid_case_returns_err() {
        let tags = build_tags(&[("addr:housenumber", "invalid??")]);
        let element_id = 6;
        let result = retrieve_housenumber_value(&tags, element_id);
        assert_err(result);
    }
}
