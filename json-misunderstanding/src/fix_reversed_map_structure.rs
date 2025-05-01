crate::ix!();

// #15 Reversed Map Structure
pub fn fix_reversed_map_structure(val: Value) -> Value {
    if let Value::Array(arr) = &val {
        let all_kv = arr.iter().all(|x| {
            x.as_object().map_or(false, |o| {
                o.contains_key("key") && o.contains_key("value")
            })
        });
        if all_kv {
            let mut new_obj = serde_json::Map::new();
            for element in arr {
                if let Value::Object(o) = element {
                    if let (Some(k_val), Some(v_val)) = (o.get("key"), o.get("value")) {
                        if let Some(k_str) = k_val.as_str() {
                            new_obj.insert(k_str.to_owned(), v_val.clone());
                        }
                    }
                }
            }
            tracing::debug!("Reversed map structure fixed by creating an object from key-value pairs");
            return Value::Object(new_obj);
        }
    }
    val
}
