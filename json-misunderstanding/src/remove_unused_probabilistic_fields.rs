crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_unused_probabilistic_fields(val: Value) -> Value {
    trace!("Starting remove_unused_probabilistic_fields");
    match val {
        Value::Object(mut obj) => {
            let mode = obj.get("mode").and_then(|m| m.as_str());
            if mode.map_or(true, |m| m != "Probabilistic") {
                if let Some(prob_val) = obj.get("probability") {
                    if let Some(f) = prob_val.as_f64() {
                        if f == 0.0 {
                            debug!("Removing probability=0.0 for non-probabilistic mode");
                            obj.remove("probability");
                            obj.remove("probability_confidence");
                            obj.remove("probability_justification");
                        }
                    }
                }
            }
            let mut final_map = serde_json::Map::new();
            for (k, v) in obj.into_iter() {
                final_map.insert(k, remove_unused_probabilistic_fields(v));
            }
            Value::Object(final_map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(remove_unused_probabilistic_fields)
                .collect(),
        ),
        other => other,
    }
}
