crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_inconsistent_nested_variant_type(val: Value) -> Value {
    trace!("Starting remove_inconsistent_nested_variant_type");
    match val {
        Value::Object(mut obj) => {
            if let Some(Value::String(vt)) = obj.get("variant_type") {
                if vt == "unit" {
                    debug!("Removing unhelpful 'variant_type': 'unit'");
                    obj.remove("variant_type");
                }
            }
            let mut final_map = serde_json::Map::new();
            for (k, v) in obj.into_iter() {
                final_map.insert(k, remove_inconsistent_nested_variant_type(v));
            }
            Value::Object(final_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(remove_inconsistent_nested_variant_type).collect())
        }
        other => other,
    }
}
