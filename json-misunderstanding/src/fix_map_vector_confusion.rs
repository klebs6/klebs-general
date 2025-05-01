crate::ix!();

pub fn fix_map_vector_confusion(val: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(obj) = &val {
        // Only trigger if there's exactly one key
        if obj.len() == 1 {
            let (single_key, potential_inner) = obj.iter().next().unwrap();
            // Check that it's not "results", and that the inner is an object with "descriptor"
            if single_key != "results" {
                if let serde_json::Value::Object(inner_map) = potential_inner {
                    if inner_map.contains_key("descriptor") {
                        tracing::debug!(
                            "fix_map_vector_confusion triggered for single key '{}'",
                            single_key
                        );
                        let mut new_obj = serde_json::Map::new();
                        new_obj.insert(
                            "name".to_owned(),
                            serde_json::Value::String(single_key.clone()),
                        );
                        // Copy the rest of the inner_map as-is
                        for (ik, iv) in inner_map {
                            new_obj.insert(ik.clone(), iv.clone());
                        }
                        return serde_json::Value::Array(vec![serde_json::Value::Object(new_obj)]);
                    }
                }
            }
        }
    }
    val
}
