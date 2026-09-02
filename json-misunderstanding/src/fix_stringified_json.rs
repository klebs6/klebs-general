crate::ix!();

/// #18 Fixes "stringified JSON" by attempting to parse any string as a nested JSON object/array.
/// Our naive strategy:
/// - If the value is a string, we try to parse it as JSON. If that succeeds, we replace it with that JSON.
pub fn fix_stringified_json(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_stringified_json");
    match val {
        serde_json::Value::String(s) => {
            let parsed = serde_json::from_str::<serde_json::Value>(&s);
            match parsed {
                Ok(nested) => {
                    debug!("Parsed stringified JSON successfully, replacing with nested object/array");
                    nested
                }
                Err(e) => {
                    trace!("String does not appear to be valid JSON: {}", e);
                    serde_json::Value::String(s)
                }
            }
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_stringified_json {
    use super::*;

    #[traced_test]
    fn test_valid_stringified_object() {
        trace!("Testing fix_stringified_json with a valid JSON string");
        let input = json!("{\"id\":1,\"valid\":true}");
        debug!("Input: {}", input);

        let expected = json!({"id":1,"valid":true});
        let output = fix_stringified_json(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Should parse valid JSON string into nested object");
        info!("Valid stringified object parsed successfully");
    }

    #[traced_test]
    fn test_invalid_json_string() {
        trace!("Testing fix_stringified_json with an invalid JSON string");
        let input = json!("not really json");
        debug!("Input: {}", input);

        let output = fix_stringified_json(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "Invalid JSON string should remain unchanged");
        info!("Invalid JSON string was left unmodified, as expected");
    }

    #[traced_test]
    fn test_non_string_input() {
        trace!("Testing fix_stringified_json with a non-string input");
        let input = json!({"key": "value"});
        debug!("Input: {}", input);

        let output = fix_stringified_json(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "Non-string inputs should remain unchanged");
        info!("Non-string input was left as-is");
    }
}
