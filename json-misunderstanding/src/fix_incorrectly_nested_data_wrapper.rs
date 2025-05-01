crate::ix!();

/// #24 Fixes "incorrectly nested data wrapper" by unwrapping {"data": [...]} -> [...]
/// if it is the top-level (naive approach).
pub fn fix_incorrectly_nested_data_wrapper(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_incorrectly_nested_data_wrapper");
    match val {
        serde_json::Value::Object(mut obj) => {
            if obj.len() == 1 && obj.contains_key("data") {
                match obj.remove("data").unwrap() {
                    serde_json::Value::Array(arr) => {
                        debug!("Unwrapping single top-level 'data' array to become top-level array");
                        return serde_json::Value::Array(arr);
                    }
                    other => {
                        trace!("Found 'data' key, but it's not an array. Leaving as-is.");
                        obj.insert("data".to_owned(), other);
                    }
                }
            }
            serde_json::Value::Object(obj)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_incorrectly_nested_data_wrapper {
    use super::*;

    #[traced_test]
    fn test_unwrap_single_data_key() {
        trace!("Testing fix_incorrectly_nested_data_wrapper with single key 'data'");
        let input = json!({"data":[{"name":"Item1"},{"name":"Item2"}]});
        debug!("Input: {}", input);

        let expected = json!([{"name":"Item1"},{"name":"Item2"}]);
        let output = fix_incorrectly_nested_data_wrapper(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Single top-level 'data' array should be unwrapped to top-level array");
        info!("Incorrectly nested data wrapper unwrapped successfully");
    }

    #[traced_test]
    fn test_data_is_not_array() {
        trace!("Testing fix_incorrectly_nested_data_wrapper with a 'data' key that's not an array");
        let input = json!({"data":{"name":"Item1"}});
        debug!("Input: {}", input);

        let output = fix_incorrectly_nested_data_wrapper(input.clone());
        debug!("Output: {}", output);

        // Because 'data' is not an array, we do nothing but keep it
        assert_eq!(output, input, "No unwrapping should occur if 'data' is not an array");
        info!("Non-array 'data' remains unchanged");
    }

    #[traced_test]
    fn test_multiple_keys() {
        trace!("Testing fix_incorrectly_nested_data_wrapper with multiple keys in object");
        let input = json!({"data":[{"name":"Item1"}], "extra":42});
        debug!("Input: {}", input);

        let output = fix_incorrectly_nested_data_wrapper(input.clone());
        debug!("Output: {}", output);

        // Because there's more than one key, we do nothing
        assert_eq!(output, input, "No unwrapping if the top-level has more keys than just 'data'");
        info!("No changes made when top-level has more than one key");
    }
}
