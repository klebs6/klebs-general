// ---------------- [ File: json-misunderstanding/src/test2.rs ]
crate::ix!();

/// This data structure represents a minimal "struct definition" in our schema.
/// In real usage, you'd have more fields, but here we demonstrate that `struct_name`
/// must be present and valid for the struct to parse successfully.
#[derive(Debug, Deserialize)]
struct SimpleStructDef {
    /// The `struct_name` field is required for all struct definitions.
    struct_name: String,

    /// Additional fields just to illustrate the concept; these could be anything
    /// your schema demands, but not strictly relevant to the test logic.
    struct_confidence: f64,
    struct_justification: String,
}

/// Attempts to parse an example struct definition from JSON. Fails if `struct_name` is missing
/// or invalid. This effectively simulates a "real scenario" where `struct_name` is mandatory.
fn parse_struct_definition(json_str: &str) -> Result<SimpleStructDef, serde_json::Error> {
    serde_json::from_str(json_str)
}

/// Repairs missing or inconsistent `struct_name` fields in any JSON object
/// that looks like a struct definition node. We identify a "struct definition"
/// if it has `"type": "struct"` OR if it contains certain hallmark fields like
/// `"struct_docs"` or `"struct_confidence"`, and we see it's missing `"struct_name"`.
/// If `struct_name` is missing or not a string, we repair it with `"RepairedStructName"`.
pub fn repair_missing_struct_name_in_struct(json_str: &str) -> Result<String, Box<dyn Error>> {
    trace!("Starting repair of missing/inconsistent `struct_name` fields in struct definitions.");

    // Parse into a generic JSON Value to allow flexible inspection and mutation.
    let mut root_value: Value = serde_json::from_str(json_str)?;
    debug!("Parsed JSON successfully. Beginning recursive repair of struct_name.");

    repair_in_value(&mut root_value);

    let repaired_str = serde_json::to_string_pretty(&root_value)?;
    info!("Repair complete. Returning repaired JSON string.");
    Ok(repaired_str)
}

/// Recursively inspects a JSON `Value` and repairs missing `struct_name` fields
/// in objects that appear to be "struct definitions."
fn repair_in_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // We consider an object a "struct definition" if it has "type": "struct"
            // or if it has a "struct_docs" or "has_justification" plus something like
            // "struct_confidence" or "struct_justification" that implies it's a struct node.
            let is_struct_type = map.get("type")
                .map(|v| v == "struct")
                .unwrap_or(false);

            let has_struct_confidence = map.get("struct_confidence")
                .map(|v| v.is_number())
                .unwrap_or(false);

            let has_struct_docs = map.get("struct_docs")
                .map(|v| v.is_string())
                .unwrap_or(false);

            // Decide if we treat this node as a "struct definition."
            let looks_like_struct = is_struct_type || (has_struct_confidence && has_struct_docs);

            if looks_like_struct {
                // Check `struct_name`. If missing or invalid, fix it.
                match map.get("struct_name") {
                    Some(val) if val.is_string() => {
                        // We have a valid struct_name, do nothing.
                    }
                    _ => {
                        warn!("Detected a missing or invalid `struct_name` in struct definition. Repairing...");
                        map.insert(
                            "struct_name".to_string(),
                            Value::String("RepairedStructName".to_owned()),
                        );
                    }
                }
            }

            // Recurse deeper into object fields
            for (_k, v) in map.iter_mut() {
                repair_in_value(v);
            }
        }
        Value::Array(arr) => {
            // Recurse into array elements
            for elem in arr.iter_mut() {
                repair_in_value(elem);
            }
        }
        _ => {
            // No action on primitives
        }
    }
}

// ----------------- TESTS -----------------

#[cfg(test)]
mod test_missing_inconsistent_struct_name {
    use super::*;

    /// Tests that parsing fails if `struct_name` is missing or invalid,
    /// and that the repair function can correct it.
    #[traced_test]
    fn test_missing_or_inconsistent_struct_name_in_struct() {
        trace!("Starting test for missing/inconsistent `struct_name` in a struct definition.");

        // Example JSON for a struct definition that omits `struct_name`.
        let invalid_struct_json = json!({
            "type": "struct",  // Indicate we have a struct
            "struct_docs": "Test documentation for demonstration.",
            "struct_confidence": 0.9,
            "struct_justification": "Demonstrating missing struct_name repair."
            // "struct_name" is missing, which should cause parse failure
        })
        .to_string();

        debug!("Attempting to parse invalid JSON:\n{}", invalid_struct_json);

        // We expect this parse to fail because `struct_name` is missing.
        let parse_result = parse_struct_definition(&invalid_struct_json);
        match parse_result {
            Ok(parsed) => {
                error!(
                    "Parse unexpectedly succeeded. Parsed: {:?}. We expected to fail due to missing `struct_name`.",
                    parsed
                );
                panic!("Test failed: JSON missing `struct_name` incorrectly parsed without error.");
            }
            Err(e) => {
                info!("Parse failed as expected due to missing `struct_name`: {}", e);
            }
        }

        // Now we'll apply the repair function to fix missing `struct_name`.
        let repaired_str = repair_missing_struct_name_in_struct(&invalid_struct_json)
            .expect("Repair function should succeed even with missing struct_name.");
        debug!("Repaired JSON output:\n{}", repaired_str);

        // Attempt to parse the repaired JSON. This time we expect success.
        let repaired_parse_result = parse_struct_definition(&repaired_str)
            .expect("Parsing the repaired JSON should succeed.");
        assert_eq!(
            repaired_parse_result.struct_name, "RepairedStructName",
            "We expect the repair to set `struct_name` = 'RepairedStructName'."
        );

        warn!("Test passed: missing `struct_name` was detected, repaired, and the repaired data parsed successfully.");
    }
}
