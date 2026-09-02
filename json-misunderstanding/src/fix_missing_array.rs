crate::ix!();

/// #17 Fixes a "missing array" scenario by converting certain comma-separated strings into arrays.
/// Our naive strategy:
/// - If the value is a string containing a comma, we split on commas into an array of trimmed strings.
pub fn fix_missing_array(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_missing_array");
    match val {
        serde_json::Value::String(ref s) => {
            if s.contains(',') {
                debug!("Detected comma-separated string; splitting into array");
                let items: Vec<_> = s
                    .split(',')
                    .map(|part| serde_json::Value::String(part.trim().to_owned()))
                    .collect();
                serde_json::Value::Array(items)
            } else {
                // No commas, so we leave it alone
                val
            }
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_missing_array {
    use super::*;

    #[traced_test]
    fn test_no_comma_in_string() {
        trace!("Testing fix_missing_array with a string that has no commas");
        let input = json!("singleTag");
        debug!("Input: {}", input);

        let output = fix_missing_array(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "String without commas should remain unchanged");
        info!("Test for no-comma string passed");
    }

    #[traced_test]
    fn test_comma_separated_string() {
        trace!("Testing fix_missing_array with a comma-separated string");
        let input = json!("rust, json, serialization");
        debug!("Input: {}", input);

        let expected = json!(["rust", "json", "serialization"]);
        let output = fix_missing_array(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Comma-separated string should become array of trimmed strings");
        info!("Test for comma-separated string -> array passed");
    }
}
