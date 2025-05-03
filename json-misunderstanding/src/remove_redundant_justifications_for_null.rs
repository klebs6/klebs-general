crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_redundant_justifications_for_null(val: Value) -> Value {
    trace!("Starting remove_redundant_justifications_for_null");
    match val {
        Value::Object(mut obj) => {
            let keys: Vec<String> = obj.keys().cloned().collect();
            for k in keys {
                if k.ends_with("_justification") {
                    let prefix = k.trim_end_matches("_justification");
                    if let Some(Value::Null) = obj.get(prefix) {
                        debug!("Removing redundant justification for null field '{}'", prefix);
                        obj.remove(&k);
                    }
                }
            }
            let mut final_map = serde_json::Map::new();
            for (kk, vv) in obj.into_iter() {
                final_map.insert(kk, remove_redundant_justifications_for_null(vv));
            }
            Value::Object(final_map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(remove_redundant_justifications_for_null)
                .collect(),
        ),
        other => other,
    }
}
