crate::ix!();

// #8 Unnecessary Additional Nesting
pub fn fix_unnecessary_additional_nesting(val: Value) -> Value {
    if let Value::Object(obj) = &val {
        if let Some(Value::Object(inner)) = obj.get("results") {
            // If there's "items" inside "results", flatten?
            if let Some(Value::Array(items)) = inner.get("items") {
                tracing::debug!("Flattening out additional nesting under 'results' -> 'items'");
                let mut new_map = serde_json::Map::new();
                // Copy everything except "results"...
                for (k, v) in obj.iter() {
                    if k != "results" {
                        new_map.insert(k.clone(), v.clone());
                    }
                }
                // Insert "results" with the array
                new_map.insert("results".to_owned(), Value::Array(items.clone()));
                return Value::Object(new_map);
            }
        }
    }
    val
}
