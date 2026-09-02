crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn fix_redundant_enum_metadata(val: Value) -> Value {
    trace!("Starting fix_redundant_enum_metadata");
    match val {
        Value::Object(obj) => {
            if let Some(mode_obj) = obj.get("mode") {
                if let Value::Object(mode_inner) = mode_obj {
                    if let Some(Value::Array(vars)) = mode_inner.get("variants") {
                        for var in vars {
                            if let Value::Object(var_obj) = var {
                                if let (Some(name_val), Some(conf_val)) = (
                                    var_obj.get("variant_name"),
                                    var_obj.get("variant_confidence"),
                                ) {
                                    let conf = conf_val.as_f64().unwrap_or(0.0);
                                    if (conf - 1.0).abs() < f64::EPSILON {
                                        if let Some(selected_name) = name_val.as_str() {
                                            debug!("Flattening redundant enum metadata for 'mode'");
                                            let mut new_obj = obj.clone();
                                            new_obj.insert("mode".to_string(), Value::String(selected_name.to_string()));
                                            return Value::Object(new_obj);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Value::Object(obj)
        }
        other => other,
    }
}
