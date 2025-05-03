crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_incorrect_type_metadata(val: Value) -> Value {
    trace!("Starting remove_incorrect_type_metadata");
    match val {
        Value::Object(mut obj) => {
            if let Some(Value::String(t)) = obj.get("type") {
                if t == "struct" || t == "complex_enum" {
                    debug!("Removing 'type' metadata from object");
                    obj.remove("type");
                }
            }
            let mut final_map = serde_json::Map::new();
            for (k, v) in obj.into_iter() {
                final_map.insert(k, remove_incorrect_type_metadata(v));
            }
            Value::Object(final_map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(remove_incorrect_type_metadata)
                .collect(),
        ),
        other => other,
    }
}
