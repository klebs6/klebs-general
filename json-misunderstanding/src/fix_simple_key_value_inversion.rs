crate::ix!();

/// #23 Fixes "key-value inversion" by swapping them if we detect that the key is "description"
/// and the value is "field_name". (Naive approach)
pub fn fix_simple_key_value_inversion(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_simple_key_value_inversion");
    match val {
        serde_json::Value::Object(mut obj) => {
            // If "description" is present and is a string that says "field_name", swap them.
            if let Some(serde_json::Value::String(s)) = obj.get("description") {
                if s == "field_name" {
                    debug!("Swapping 'description' <-> 'field_name'");
                    // Remove old entry
                    obj.remove("description");
                    // Insert new
                    obj.insert("field_name".to_owned(), serde_json::Value::String("description".to_owned()));
                }
            }
            serde_json::Value::Object(obj)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_simple_key_value_inversion {
    use super::*;

    #[traced_test]
    fn test_description_inverted() {
        trace!("Testing fix_simple_key_value_inversion with 'description' key having 'field_name' as value");
        let input = json!({"description": "field_name"});
        debug!("Input: {}", input);

        let expected = json!({"field_name": "description"});
        let output = fix_simple_key_value_inversion(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Should swap 'description' <-> 'field_name'");
        info!("Key-value inversion swapped successfully");
    }

    #[traced_test]
    fn test_unrelated_object() {
        trace!("Testing fix_simple_key_value_inversion with unrelated object");
        let input = json!({"description": "example", "other": 123});
        debug!("Input: {}", input);

        let output = fix_simple_key_value_inversion(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "No swap because value is not 'field_name'");
        info!("Unrelated object remains unchanged");
    }
}
