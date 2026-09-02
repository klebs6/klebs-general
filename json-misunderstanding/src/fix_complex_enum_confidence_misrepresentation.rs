crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn fix_complex_enum_confidence_misrepresentation(val: Value) -> Value {
    trace!("Starting fix_complex_enum_confidence_misrepresentation");
    match &val {
        Value::Object(obj) => {
            if obj.get("type").map_or(false, |t| t == "complex_enum")
                && obj.get("enum_name").is_some()
                && obj.get("variants").map_or(false, |v| v.is_array())
            {
                if let Some(Value::Array(variants)) = obj.get("variants") {
                    let mut selected_variant = None;
                    let mut selected_confidence = None;
                    let mut selected_justification = None;

                    for variant_val in variants {
                        if let Value::Object(variant_obj) = variant_val {
                            let conf = variant_obj
                                .get("variant_confidence")
                                .and_then(|c| c.as_f64())
                                .unwrap_or(0.0);
                            if conf > 0.0 {
                                // If we already have a variant with positive confidence, we bail and leave unchanged
                                if selected_variant.is_some() {
                                    trace!("Multiple variants with confidence > 0. Leaving as-is.");
                                    return val;
                                }
                                let name = variant_obj
                                    .get("variant_name")
                                    .and_then(|n| n.as_str())
                                    .map(|s| s.to_string());
                                let just = variant_obj
                                    .get("variant_justification")
                                    .and_then(|n| n.as_str())
                                    .map(|s| s.to_string());

                                if let Some(nm) = name {
                                    selected_variant = Some(nm);
                                    selected_confidence = Some(conf);
                                    selected_justification = just;
                                }
                            }
                        }
                    }

                    if let (Some(var), Some(conf)) = (selected_variant, selected_confidence) {
                        debug!("Transforming complex enum to single-variant with confidence");
                        let mut new_obj = serde_json::Map::new();
                        new_obj.insert("variant".to_string(), Value::String(var));
                        
                        // Convert f64 -> Number safely
                        if let Some(num) = serde_json::Number::from_f64(conf) {
                            new_obj.insert("confidence".to_string(), Value::Number(num));
                        } else {
                            // If from_f64 fails (NaN/infinite), fallback
                            new_obj.insert("confidence".to_string(), Value::Null);
                        }

                        if let Some(just) = selected_justification {
                            new_obj.insert("justification".to_string(), Value::String(just));
                        }
                        return Value::Object(new_obj);
                    }
                }
            }
        }
        _ => {}
    }
    val
}
