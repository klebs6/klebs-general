crate::ix!();

/// #19 Fixes "misplaced metadata" by moving known 'metadata' fields into a separate 'metadata' object.
/// Naive approach:
/// - If we see top-level fields that look like they might be metadata (like "timestamp"),
///   and also see at least one "data-like" field (like "item" or "data"), we move them.
pub fn fix_misplaced_metadata(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_misplaced_metadata");
    match val {
        serde_json::Value::Object(mut obj) => {
            // Heuristic: if there's an obvious data field ("item", "data", "payload"), we treat the rest as metadata
            let has_data_field = obj.contains_key("item") || obj.contains_key("data") || obj.contains_key("payload");

            // We'll collect any known metadata keys
            let known_metadata_keys = vec!["timestamp", "author", "version", "meta"];
            let mut metadata_map = serde_json::Map::new();
            let mut to_remove = vec![];

            if has_data_field {
                for (k, v) in obj.iter() {
                    if known_metadata_keys.contains(&k.as_str()) {
                        debug!("Moving '{}' into a 'metadata' subobject", k);
                        metadata_map.insert(k.clone(), v.clone());
                        to_remove.push(k.clone());
                    }
                }
            }

            // Remove those from the original object
            for k in to_remove {
                obj.remove(&k);
            }

            if !metadata_map.is_empty() && !obj.contains_key("metadata") {
                info!("Created a new 'metadata' subobject for misplaced metadata fields");
                obj.insert("metadata".to_owned(), serde_json::Value::Object(metadata_map));
            }

            serde_json::Value::Object(obj)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_misplaced_metadata {
    use super::*;

    #[traced_test]
    fn test_metadata_already_separated() {
        trace!("Testing fix_misplaced_metadata where data and metadata are already separate");
        let input = json!({"data":{"item":"A"},"metadata":{"timestamp":123456}});
        debug!("Input: {}", input);

        let output = fix_misplaced_metadata(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "Already-separated metadata should remain as-is");
        info!("No changes applied when data is already separated from metadata");
    }

    #[traced_test]
    fn test_move_metadata_to_subobject() {
        trace!("Testing fix_misplaced_metadata with 'item' and 'timestamp' at top-level");
        let input = json!({"item":"A","timestamp":123456});
        debug!("Input: {}", input);

        // We expect "item" to remain, "timestamp" to be moved into "metadata" if we consider "item" as data.
        let output = fix_misplaced_metadata(input.clone());
        debug!("Output: {}", output);

        assert!(output.get("item").is_some(), "Should keep 'item' at top-level");
        let metadata = output.get("metadata").and_then(|m| m.as_object());
        assert!(metadata.is_some(), "Should have a 'metadata' object");
        assert_eq!(
            metadata.unwrap().get("timestamp"),
            Some(&json!(123456)),
            "Timestamp should be inside 'metadata'"
        );
        info!("Metadata fields were moved into a 'metadata' subobject successfully");
    }
}
