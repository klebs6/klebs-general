// ---------------- [ File: json-misunderstanding/src/test1.rs ]
crate::ix!();

/// Repairs missing or inconsistent `variant_name` fields in any JSON object 
/// that appears to be an enum variant node. We detect such nodes by checking 
/// if they contain `variant_confidence` (number) and `variant_justification` (string), 
/// which typically accompany a valid `variant_name`. If `variant_name` is missing 
/// or not a string, we replace it with "RepairedVariantName".
pub fn repair_missing_variant_name_in_enum(json_str: &str) -> Result<String, Box<dyn Error>> {
    trace!("Starting repair of missing/inconsistent `variant_name` fields.");

    // Parse the incoming JSON into a serde_json::Value for flexible inspection.
    let mut root_value: Value = serde_json::from_str(json_str)?;
    debug!("Parsed JSON successfully. Beginning recursive repair procedure.");

    // Recursively walk the JSON structure, fixing missing or inconsistent `variant_name`.
    repair_in_value(&mut root_value);

    // Convert repaired JSON back to a string for output.
    let repaired_str = serde_json::to_string_pretty(&root_value)?;
    info!("Repair complete. Returning repaired JSON string.");
    Ok(repaired_str)
}

/// Recursively inspects a JSON `Value` and repairs missing/inconsistent `variant_name` fields
/// in objects that appear to be enum variant definitions.
fn repair_in_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Check if this object qualifies as an enum variant node:
            // Must have "variant_confidence" (number) and "variant_justification" (string).
            let has_variant_conf = map.get("variant_confidence").map(|v| v.is_number()).unwrap_or(false);
            let has_variant_just = map.get("variant_justification").map(|v| v.is_string()).unwrap_or(false);

            // If it looks like an enum variant node, ensure "variant_name" exists and is a string.
            if has_variant_conf && has_variant_just {
                match map.get("variant_name") {
                    Some(val) if val.is_string() => {
                        // variant_name is present and valid string. No action needed.
                    }
                    _ => {
                        warn!("Detected missing or invalid `variant_name` in an enum variant. Repairing...");
                        map.insert("variant_name".to_string(), Value::String("RepairedVariantName".to_owned()));
                    }
                }
            }

            // Recurse into all child values in this object to ensure deeper fixes if needed.
            for (_k, v) in map.iter_mut() {
                repair_in_value(v);
            }
        }
        Value::Array(arr) => {
            // Recurse into array elements.
            for elem in arr.iter_mut() {
                repair_in_value(elem);
            }
        }
        _ => {
            // Primitives (string, number, bool, null) have no children, so do nothing.
        }
    }
}

// ------------------- TESTS -------------------

#[cfg(test)]
mod test_repair_missing_variant_name_in_enum {
    use super::*;

    /// Demonstrates a JSON snippet missing `variant_name` in a node
    /// that otherwise qualifies as an enum variant object.
    #[traced_test]
    fn test_repair_missing_or_inconsistent_variant_name_in_enum() {
        trace!("Testing repair of missing `variant_name` in an enum-like object.");

        let invalid_json = json!({
            "enum_name": "ExampleEnum",
            "variants": [
                {
                    // Missing variant_name, but includes fields typical of an enum variant
                    "variant_confidence": 0.9,
                    "variant_justification": "Intentional test of missing field"
                },
                {
                    // This entry is already valid
                    "variant_name": "ValidVariant",
                    "variant_confidence": 0.8,
                    "variant_justification": "A properly specified variant"
                }
            ]
        })
        .to_string();

        debug!("Original invalid JSON: {}", invalid_json);

        let repaired_result = repair_missing_variant_name_in_enum(&invalid_json)
            .expect("Repair function should succeed even with missing variant_name.");

        debug!("Repaired JSON output:\n{}", repaired_result);

        // Check that the repaired JSON indeed has a valid "variant_name" 
        // for the previously invalid entry.
        let repaired_value: serde_json::Value = serde_json::from_str(&repaired_result).unwrap();
        let variants = repaired_value.get("variants").unwrap().as_array().unwrap();

        // The first variant was missing `variant_name`; we expect it to be "RepairedVariantName" now.
        let first_variant = variants[0].as_object().unwrap();
        let repaired_name = first_variant.get("variant_name").unwrap().as_str().unwrap();

        assert_eq!(repaired_name, "RepairedVariantName");

        info!("Test passed: missing variant_name was correctly repaired to 'RepairedVariantName'.");
    }
}
