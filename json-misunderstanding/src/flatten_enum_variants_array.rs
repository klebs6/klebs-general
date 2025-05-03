crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn flatten_enum_variants_array(val: Value) -> Value {
    trace!("Starting flatten_enum_variants_array");
    match val {
        Value::Object(obj) => {
            if let (Some(Value::String(ty)), Some(Value::Array(vars))) = (obj.get("type"), obj.get("variants")) {
                if ty == "complex_enum" && !vars.is_empty() {
                    let mut best_conf = -1.0;
                    let mut best_variant: Option<String> = None;
                    let mut best_fields: Option<Value> = None;
                    for v in vars {
                        if let Value::Object(vobj) = v {
                            let c = vobj.get("variant_confidence").and_then(|x| x.as_f64()).unwrap_or(0.0);
                            if c > best_conf {
                                best_conf = c;
                                best_variant = vobj.get("variant_name").and_then(|x| x.as_str()).map(|s| s.to_string());
                                best_fields = vobj.get("fields").cloned();
                            }
                        }
                    }
                    if let Some(var_name) = best_variant {
                        debug!("Flattening array of variants to single variant '{}'", var_name);
                        let mut single = serde_json::Map::new();
                        if let Some(fields) = best_fields {
                            single.insert(var_name, fields);
                        } else {
                            single.insert(var_name, json!({}));
                        }
                        return Value::Object(single);
                    }
                }
            }
            let mut new_map = serde_json::Map::new();
            for (k, v) in obj.into_iter() {
                new_map.insert(k, flatten_enum_variants_array(v));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(flatten_enum_variants_array).collect())
        }
        other => other,
    }
}
