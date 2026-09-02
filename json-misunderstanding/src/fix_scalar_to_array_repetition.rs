crate::ix!();

pub fn fix_scalar_to_array_repetition(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_scalar_to_array_repetition");
    match val {
        serde_json::Value::Object(mut obj) => {
            // Naive rule: treat keys ending in 's' as intended arrays.
            let keys: Vec<String> = obj.keys().cloned().collect();
            for key in keys {
                // Remove the original value so we can reinsert wrapped or as-is.
                let value = obj.remove(&key).unwrap();
                if key.ends_with('s') && !matches!(value, serde_json::Value::Array(_)) {
                    debug!("Wrapping scalar in an array for key '{}'", key);
                    obj.insert(key, serde_json::Value::Array(vec![value]));
                } else {
                    // Reinsert the original value unmodified
                    obj.insert(key, value);
                }
            }
            serde_json::Value::Object(obj)
        }
        // Non-object values remain unchanged
        other => other,
    }
}

#[cfg(test)]
mod test_fix_scalar_to_array_repetition {
    use super::*;

    #[traced_test]
    fn test_wrap_scalar_in_array() {
        trace!("Testing fix_scalar_to_array_repetition for a key that ends in 's'");
        let input = json!({
            "levels": 1,
            "items": ["already","array"],
            "count": 42
        });
        debug!("Input: {}", input);

        let expected = json!({
            "levels": [1],
            "items": ["already","array"],
            "count": 42
        });
        let output = fix_scalar_to_array_repetition(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Scalar for 'levels' key should be wrapped in an array");
        info!("Scalar-to-array repetition fix applied for keys ending with 's'");
    }

    #[traced_test]
    fn test_already_an_array() {
        trace!("Testing fix_scalar_to_array_repetition with an already-array");
        let input = json!({"users": ["alice", "bob"]});
        debug!("Input: {}", input);

        let output = fix_scalar_to_array_repetition(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "Keys ending with 's' that are already arrays remain unchanged");
        info!("No changes for fields that are already an array");
    }

    #[traced_test]
    fn test_key_not_ending_in_s() {
        trace!("Testing fix_scalar_to_array_repetition with a key not ending in 's'");
        let input = json!({"level": 5});
        debug!("Input: {}", input);

        let output = fix_scalar_to_array_repetition(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "No array wrapping if key does not end with 's'");
        info!("Key not recognized as plural remains a scalar");
    }
}
