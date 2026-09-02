crate::ix!();

pub fn fix_mixed_type_arrays(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_mixed_type_arrays");
    match val {
        serde_json::Value::Array(arr) => {
            // Determine if this array is "mixed-type."
            // We'll gather each item's "broad type" (object/array/string/number/bool/null).
            // If there's more than one distinct type, it's "mixed."
            let mut types_encountered = std::collections::HashSet::new();
            for item in &arr {
                let t = match item {
                    serde_json::Value::Object(_) => "object",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::Bool(_)   => "bool",
                    serde_json::Value::Null      => "null",
                };
                types_encountered.insert(t);
                if types_encountered.len() > 1 {
                    break;
                }
            }
            let is_mixed_type = (types_encountered.len() > 1);

            let mut changed = false;
            let fixed_arr: Vec<serde_json::Value> = arr
                .into_iter()
                .map(|item| {
                    match item {
                        // If we have an object with exactly one key "value", flatten it unconditionally.
                        serde_json::Value::Object(obj) if obj.len() == 1 && obj.contains_key("value") => {
                            debug!("Flattening object with single 'value' key in a mixed-type array");
                            changed = true;
                            obj.get("value").cloned().unwrap_or(serde_json::Value::Null)
                        }
                        // If the array is truly mixed-type, we also flatten any single-element sub-array.
                        serde_json::Value::Array(mut inner_arr) 
                            if is_mixed_type && inner_arr.len() == 1 => 
                        {
                            debug!("Flattening single-element array in a mixed-type array");
                            changed = true;
                            inner_arr.pop().unwrap()
                        }
                        other => other,
                    }
                })
                .collect();

            if changed {
                info!("Mixed-type array items were homogenized into simpler forms");
            }
            serde_json::Value::Array(fixed_arr)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_mixed_type_arrays {
    use super::*;

    #[traced_test]
    fn test_already_homogeneous_array() {
        trace!("Testing fix_mixed_type_arrays on an already-homogeneous array");
        let input = json!(["val1", "val2", "val3"]);
        debug!("Input: {}", input);

        let output = fix_mixed_type_arrays(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "Homogeneous array should remain unchanged");
        info!("Test for homogeneous array passed");
    }

    #[traced_test]
    fn test_mixed_array_with_object_value_field() {
        trace!("Testing fix_mixed_type_arrays with an array containing an object with a single 'value' key");
        let input = json!(["val1", {"value": "val2"}, ["val3"]]);
        debug!("Input: {}", input);

        let expected = json!(["val1", "val2", "val3"]);
        let output = fix_mixed_type_arrays(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Should flatten single 'value' key objects and single-element arrays");
        info!("Mixed-type array with single 'value' object and single-element array was flattened successfully");
    }

    #[traced_test]
    fn test_mixed_array_no_fix_needed() {
        trace!("Testing fix_mixed_type_arrays where no fix is triggered");
        let input = json!(["val1", {"actual": "object"}, ["arr1", "arr2"]]);
        debug!("Input: {}", input);

        let output = fix_mixed_type_arrays(input.clone());
        debug!("Output: {}", output);

        // We expect the same structure because we only flatten if there's exactly one 'value' key
        // or a single-element array. This object has "actual" key, not "value", and the sub-array
        // has more than one element.
        assert_eq!(output, input, "No flattening should occur given the naive approach");
        info!("No changes applied when data doesn't match flattening criteria");
    }
}
