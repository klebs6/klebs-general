crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn remove_struct_level_just_conf(val: Value) -> Value {
    trace!("Starting remove_struct_level_just_conf");
    match val {
        Value::Object(mut obj) => {
            let keys: Vec<String> = obj.keys().cloned().collect();
            for k in keys {
                if k.ends_with("_confidence") || k.ends_with("_justification") {
                    let candidate_field = k.trim_end_matches("_confidence")
                                           .trim_end_matches("_justification");
                    // If neither the plain field nor any sub-object is named candidate_field, 
                    // assume it's "struct-level".
                    if !obj.contains_key(candidate_field) {
                        debug!("Removing struct-level confidence/justification '{}'", k);
                        obj.remove(&k);
                    }
                }
            }
            let mut final_map = serde_json::Map::new();
            for (kk, vv) in obj.into_iter() {
                final_map.insert(kk, remove_struct_level_just_conf(vv));
            }
            Value::Object(final_map)
        }
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(remove_struct_level_just_conf)
                .collect(),
        ),
        other => other,
    }
}
