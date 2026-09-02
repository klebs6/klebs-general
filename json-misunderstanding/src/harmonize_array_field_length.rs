crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn harmonize_array_field_length(val: Value) -> Value {
    trace!("Starting harmonize_array_field_length");
    match &val {
        Value::Object(obj) => {
            let depth_opt = obj.get("depth").and_then(|d| d.as_i64());
            if let Some(d) = depth_opt {
                if let Some(Value::Array(density)) = obj.get("density_per_level") {
                    if (density.len() as i64) != d {
                        debug!("Adjusting 'density_per_level' to match 'depth' = {}", d);
                        let mut new_density = density.clone();
                        if (new_density.len() as i64) < d {
                            while (new_density.len() as i64) < d {
                                new_density.push(json!(0));
                            }
                        } else {
                            new_density.truncate(d as usize);
                        }
                        let mut new_obj = obj.clone();
                        new_obj.insert("density_per_level".to_string(), Value::Array(new_density));
                        return Value::Object(new_obj);
                    }
                }
            }
        }
        _ => {}
    }
    val
}
