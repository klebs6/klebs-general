crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn fix_unit_enum_variants_wrapped_as_objects(val: Value) -> Value {
    trace!("Starting fix_unit_enum_variants_wrapped_as_objects");
    match &val {
        Value::Object(obj)
            if obj.get("type").map_or(false, |t| t == "complex_enum")
                && obj.get("variants").map_or(false, |v| v.is_array()) =>
        {
            if let Some(Value::Array(variants)) = obj.get("variants") {
                let mut chosen = None;
                for variant_val in variants {
                    if let Value::Object(vo) = variant_val {
                        let conf = vo
                            .get("variant_confidence")
                            .and_then(|c| c.as_f64())
                            .unwrap_or(0.0);
                        if (conf - 1.0).abs() < f64::EPSILON {
                            if let Some(name) = vo
                                .get("variant_name")
                                .and_then(|n| n.as_str())
                            {
                                chosen = Some(name.to_string());
                                break;
                            }
                        }
                    }
                }
                if let Some(chosen_variant) = chosen {
                    debug!("Flattening unit enum variant to a simple string '{}'", chosen_variant);
                    return Value::String(chosen_variant);
                }
            }
        }
        _ => {}
    }
    val
}
