crate::ix!();

pub fn fix_enumeration_as_map(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_enumeration_as_map");
    match val {
        serde_json::Value::Object(mut obj) => {
            for (key, value) in obj.iter_mut() {
                if let serde_json::Value::Object(inner_map) = value {
                    // If there's exactly one key in the inner map:
                    if inner_map.len() == 1 {
                        let (enum_variant, variant_val) = inner_map.iter().next().unwrap();
                        // Only flatten if that variant_val is an empty object
                        if let Some(inner_obj) = variant_val.as_object() {
                            if inner_obj.is_empty() {
                                debug!("Flattening enumeration map for '{}': '{}'", key, enum_variant);
                                *value = serde_json::Value::String(enum_variant.clone());
                            }
                        }
                    }
                }
            }
            serde_json::Value::Object(obj)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_enumeration_as_map {
    use super::*;

    #[traced_test]
    fn test_already_string_variant() {
        trace!("Testing fix_enumeration_as_map on an already-correct enumeration");
        let input = json!({"status":"Success"});
        debug!("Input: {}", input);

        let output = fix_enumeration_as_map(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "Already-correct enumeration should remain unchanged");
        info!("No changes for already-correct enumerations");
    }

    #[traced_test]
    fn test_map_to_enum_variant() {
        trace!("Testing fix_enumeration_as_map with an object variant");
        let input = json!({"status": {"Success": {}}});
        debug!("Input: {}", input);

        let expected = json!({"status":"Success"});
        let output = fix_enumeration_as_map(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Enum map structure should be converted to a string variant");
        info!("Map-based enum was flattened to a string variant successfully");
    }

    #[traced_test]
    fn test_map_with_non_empty_variant() {
        trace!("Testing fix_enumeration_as_map with a non-empty object variant");
        let input = json!({"status": {"Error": {"details":"something"}}});
        debug!("Input: {}", input);

        // Because our naive approach only flattens if the variant object is empty, this remains unchanged.
        let output = fix_enumeration_as_map(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "We do not flatten if the variant object is not empty");
        info!("Map-based enum with non-empty subobject remains unchanged in naive approach");
    }
}
