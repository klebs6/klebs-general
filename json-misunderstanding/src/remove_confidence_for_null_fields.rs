crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_confidence_for_null_fields(val: Value) -> Value {
    trace!("Starting remove_confidence_for_null_fields");
    match val {
        Value::Object(mut obj) => {
            let keys: Vec<String> = obj.keys().cloned().collect();
            for k in keys {
                if k.ends_with("_confidence") {
                    let prefix = k.trim_end_matches("_confidence");
                    if !obj.contains_key(prefix) || matches!(obj.get(prefix), Some(Value::Null)) {
                        debug!("Removing confidence field '{}' for null/omitted '{}'", k, prefix);
                        obj.remove(&k);
                    }
                }
            }
            let mut final_map = serde_json::Map::new();
            for (kk, vv) in obj.into_iter() {
                final_map.insert(kk, remove_confidence_for_null_fields(vv));
            }
            Value::Object(final_map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(remove_confidence_for_null_fields)
                .collect(),
        ),
        other => other,
    }
}
