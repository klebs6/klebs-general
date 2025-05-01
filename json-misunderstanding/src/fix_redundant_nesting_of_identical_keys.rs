crate::ix!();

/// Replaces repeated nesting like {"config": {"config": {...}}} by merging
/// the nested subobject into the parent. But the old approach only worked
/// if the nested subobject was the *only* key. The test wants multi-layer
/// merges even if the subobject has additional keys. So we do a custom "merge"
/// approach: if an object has a subobject that includes a key matching the
/// parent's key, we remove that matching subobject, then merge it up.
#[tracing::instrument(level="trace", skip_all)]
pub fn fix_redundant_nesting_of_identical_keys(val: serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::Object(obj) => {
            // Recurse children first
            let mut obj = obj
                .into_iter()
                .map(|(k, v)| (k, fix_redundant_nesting_of_identical_keys(v)))
                .collect::<serde_json::Map<_, _>>();

            let mut changed = true;
            while changed {
                changed = false;

                // We'll build a new map in each pass
                let mut new_map = serde_json::Map::new();
                for (key, value) in obj.into_iter() {
                    match value {
                        serde_json::Value::Object(mut child_obj) => {
                            // If child_obj has the same key 'key', we *merge*
                            // that nested object’s fields into child_obj,
                            // then skip that level of nesting entirely.
                            if let Some(nested_val) = child_obj.remove(&key) {
                                if let serde_json::Value::Object(nested_map) = nested_val {
                                    debug!("Flattening repeated '{}' by merging subobject up", key);
                                    // Merge nested_map into child_obj
                                    for (nk, nv) in nested_map {
                                        child_obj.insert(nk, nv);
                                    }
                                    changed = true;
                                } else {
                                    // If child_obj[key] wasn't an object,
                                    // we do nothing special
                                }
                            }
                            new_map.insert(key, serde_json::Value::Object(child_obj));
                        }
                        other => {
                            new_map.insert(key, other);
                        }
                    }
                }
                obj = new_map;
            }

            serde_json::Value::Object(obj)
        }
        serde_json::Value::Array(arr) => {
            let fixed_arr = arr
                .into_iter()
                .map(fix_redundant_nesting_of_identical_keys)
                .collect::<Vec<_>>();
            serde_json::Value::Array(fixed_arr)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_redundant_nesting_of_identical_keys {
    use super::*;

    #[traced_test]
    fn test_redundant_nesting_multiple_layers() {
        trace!("Testing fix_redundant_nesting_of_identical_keys with multiple layers");
        let input = json!({
            "config": {
                "config": {
                    "timeout": 30,
                    "config": {
                        "config": {
                            "another": true
                        }
                    }
                }
            }
        });
        debug!("Input: {}", input);

        let expected = json!({
            "config": {
                "timeout": 30,
                "another": true
            }
        });
        let output = fix_redundant_nesting_of_identical_keys(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Should repeatedly flatten nested identical keys");
        info!("Redundant nesting of identical keys flattened across multiple layers");
    }

    #[traced_test]
    fn test_no_redundant_nesting() {
        trace!("Testing fix_redundant_nesting_of_identical_keys with no redundant nesting");
        let input = json!({"config": {"timeout": 30}});
        debug!("Input: {}", input);

        let output = fix_redundant_nesting_of_identical_keys(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "No changes when there's no repeated nesting of same key");
        info!("No flattening applied when structure is already correct");
    }
}
