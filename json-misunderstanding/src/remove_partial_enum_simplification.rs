crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn resolve_partial_enum_simplification(val: Value) -> Value {
    trace!("Starting resolve_partial_enum_simplification");
    match val {
        Value::Object(obj) => {
            if obj.contains_key("variant_name") && obj.len() > 1 {
                if let Some(Value::Number(conf)) = obj.get("variant_confidence") {
                    if conf.as_f64().unwrap_or(0.0) >= 1.0 {
                        if let Some(Value::String(vn)) = obj.get("variant_name") {
                            debug!("Converting partial enum to a fully simplified string variant");
                            return Value::String(vn.clone());
                        }
                    }
                }
            }
            let mut new_map = serde_json::Map::new();
            for (k, v) in obj.into_iter() {
                new_map.insert(k, resolve_partial_enum_simplification(v));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(
                arr.into_iter()
                    .map(resolve_partial_enum_simplification)
                    .collect(),
            )
        }
        other => other,
    }
}
