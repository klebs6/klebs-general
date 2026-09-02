crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn pull_up_nested_just_conf_fields(val: Value) -> Value {
    trace!("Starting pull_up_nested_just_conf_fields");
    match val {
        Value::Object(mut obj) => {
            // We first recurse deeper to ensure child objects are fixed first:
            for (k, v) in obj.iter_mut() {
                let fixed_child = pull_up_nested_just_conf_fields(std::mem::take(v));
                *v = fixed_child;
            }

            // Now we scan for any field `k` whose value is an object containing { value, confidence?, justification? }.
            // We remove the original `k` entry, then re-insert top-level "k" with the `value`, and (optionally)
            // "k_confidence" with `confidence`, and "k_justification" with `justification`.
            //
            // This avoids the issue where inserting k before removing it caused the final remove to delete it.
            let keys: Vec<String> = obj.keys().cloned().collect();
            for k in keys {
                let current_val = obj.get(&k).cloned().unwrap_or(Value::Null);
                if let Value::Object(inner_map) = current_val {
                    let has_value = inner_map.get("value");
                    let has_conf = inner_map.get("confidence");
                    let has_just = inner_map.get("justification");

                    // We only do the "pull up" if there's a `value` plus at least confidence or justification.
                    // (At minimum, we do want to flatten if there's a 'value'.)
                    if has_value.is_some() && (has_conf.is_some() || has_just.is_some()) {
                        debug!("Pulling up justification/confidence from child object for key '{}'", k);

                        // Remove original entry to avoid overwriting issues
                        obj.remove(&k);

                        // Insert the main field if there's a `value`
                        if let Some(val_node) = has_value {
                            obj.insert(k.clone(), val_node.clone());
                        }

                        // Insert the confidence field if present
                        if let Some(conf_node) = has_conf {
                            obj.insert(format!("{}_confidence", k), conf_node.clone());
                        }

                        // Insert the justification field if present
                        if let Some(just_node) = has_just {
                            obj.insert(format!("{}_justification", k), just_node.clone());
                        }
                    } else {
                        // If we don't meet the "flatten" criteria, put it back as is
                        obj.insert(k, Value::Object(inner_map));
                    }
                }
            }

            Value::Object(obj)
        }
        Value::Array(arr) => {
            // Recurse into array elements
            let fixed_arr = arr
                .into_iter()
                .map(pull_up_nested_just_conf_fields)
                .collect();
            Value::Array(fixed_arr)
        }
        other => other,
    }
}
