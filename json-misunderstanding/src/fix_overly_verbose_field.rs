crate::ix!();

/// #25 Fixes "overly verbose field" by replacing {"count": {"value": 3}} with {"count": 3}
/// if an object has a single key "value".
pub fn fix_overly_verbose_field(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_overly_verbose_field");
    match val {
        serde_json::Value::Object(mut obj) => {
            // For each key -> if that key's value is an object with single key "value", flatten it.
            let mut to_update = Vec::new();
            for (k, v) in obj.iter() {
                if let serde_json::Value::Object(inner_map) = v {
                    if inner_map.len() == 1 && inner_map.contains_key("value") {
                        to_update.push(k.clone());
                    }
                }
            }
            for k in to_update {
                if let serde_json::Value::Object(inner_map) = obj.remove(&k).unwrap() {
                    debug!("Flattening overly verbose field '{}':{{'value':..}} -> direct scalar", k);
                    if let Some(new_val) = inner_map.get("value") {
                        obj.insert(k, new_val.clone());
                    }
                }
            }
            serde_json::Value::Object(obj)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_overly_verbose_field {
    use super::*;

    #[traced_test]
    fn test_flatten_overly_verbose_field() {
        trace!("Testing fix_overly_verbose_field with a field that has object {{ value: X }}");
        let input = json!({
            "count": {"value": 3},
            "other": {"value": 10, "extra": true},
            "plain": 42
        });
        debug!("Input: {}", input);

        let expected = json!({
            "count": 3,
            "other": {"value": 10, "extra": true},
            "plain": 42
        });
        let output = fix_overly_verbose_field(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Should flatten single-key 'value' objects for 'count' only");
        info!("Flattened overly verbose field successfully for 'count'");
    }

    #[traced_test]
    fn test_nothing_to_flatten() {
        trace!("Testing fix_overly_verbose_field where no field is overly verbose");
        let input = json!({"count": 3, "label": {"key": 1, "value": 2}});
        debug!("Input: {}", input);

        let output = fix_overly_verbose_field(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "No single-key 'value' fields to flatten");
        info!("No changes when there's nothing to flatten");
    }
}
