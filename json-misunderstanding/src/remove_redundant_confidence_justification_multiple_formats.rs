crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_redundant_confidence_justification_multiple_levels(val: Value) -> Value {
    trace!("Starting remove_redundant_confidence_justification_multiple_levels");
    match val {
        Value::Object(mut obj) => {
            let keys: Vec<String> = obj.keys().cloned().collect();
            for k in keys {
                if k.ends_with("_confidence") || k.ends_with("_justification") {
                    let prefix = if k.ends_with("_confidence") {
                        k.trim_end_matches("_confidence")
                    } else {
                        k.trim_end_matches("_justification")
                    };
                    if let Some(child_val) = obj.get(prefix) {
                        if child_val.is_object() {
                            debug!("Removing outer-level justification/confidence '{}' due to child object for '{}'", k, prefix);
                            obj.remove(&k);
                        }
                    }
                }
            }
            let mut final_map = serde_json::Map::new();
            for (kk, vv) in obj.into_iter() {
                final_map.insert(kk, remove_redundant_confidence_justification_multiple_levels(vv));
            }
            Value::Object(final_map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(remove_redundant_confidence_justification_multiple_levels)
                .collect(),
        ),
        other => other,
    }
}
