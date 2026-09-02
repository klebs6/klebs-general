crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_confidence_from_non_selected_variants(val: Value) -> Value {
    trace!("Starting remove_confidence_from_non_selected_variants");
    match val {
        Value::Object(mut obj) => {
            if let Some(Value::Array(vars)) = obj.get("variants") {
                let new_vars: Vec<Value> = vars
                    .iter()
                    .filter_map(|v| {
                        if let Value::Object(vobj) = v {
                            let c = vobj.get("variant_confidence").and_then(|x| x.as_f64()).unwrap_or(0.0);
                            if c > 0.0 {
                                Some(Value::Object(vobj.clone()))
                            } else {
                                None
                            }
                        } else {
                            Some(v.clone())
                        }
                    })
                    .collect();
                let mut changed_obj = obj.clone();
                changed_obj.insert("variants".to_string(), Value::Array(new_vars));
                Value::Object(changed_obj)
            } else {
                let mut final_map = serde_json::Map::new();
                for (k, v) in obj.into_iter() {
                    final_map.insert(k, remove_confidence_from_non_selected_variants(v));
                }
                Value::Object(final_map)
            }
        }
        Value::Array(arr) => {
            Value::Array(
                arr.into_iter()
                    .map(remove_confidence_from_non_selected_variants)
                    .collect(),
            )
        }
        other => other,
    }
}
