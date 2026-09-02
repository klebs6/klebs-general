crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_schema_metadata_in_output(val: Value) -> Value {
    trace!("Starting remove_schema_metadata_in_output");
    match val {
        Value::Object(mut obj) => {
            let removal_keys = &["generation_instructions","required","type"];
            for rk in removal_keys {
                if obj.contains_key(*rk) {
                    debug!("Removing schema metadata field '{}'", rk);
                    obj.remove(*rk);
                }
            }
            if let Some(val_sub) = obj.get("value") {
                let has_meta = removal_keys.iter().any(|m| obj.contains_key(*m));
                if has_meta || obj.len() == 1 {
                    debug!("Flattening object that has only 'value' plus schema metadata");
                    return val_sub.clone();
                }
            }
            let mut final_map = serde_json::Map::new();
            for (kk, vv) in obj.into_iter() {
                final_map.insert(kk, remove_schema_metadata_in_output(vv));
            }
            Value::Object(final_map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(remove_schema_metadata_in_output)
                .collect(),
        ),
        other => other,
    }
}
