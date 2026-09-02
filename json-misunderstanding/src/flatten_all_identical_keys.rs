crate::ix!();

/// This helper keeps applying the "flatten repeated K inside K" pattern 
/// until it is fully resolved. We also recurse child objects at each step.
#[tracing::instrument(level="trace", skip_all)]
pub fn flatten_all_identical_keys(mut val: serde_json::Value) -> serde_json::Value {
    loop {
        let old = val.clone();
        val = flatten_one_level(old.clone());
        if val == old {
            break;
        }
    }
    val
}

/// Performs ONE pass of "if object has {K: {K: subval}}, flatten that in place."
#[tracing::instrument(level="trace", skip_all)]
pub fn flatten_one_level(val: serde_json::Value) -> serde_json::Value {
    let mut obj = match val {
        serde_json::Value::Object(o) => o,
        other => return other,
    };

    // First, recurse children:
    for (k, v) in obj.iter_mut() {
        let child_fixed = fix_redundant_nesting_of_identical_keys(std::mem::take(v));
        *v = child_fixed;
    }

    // Then flatten any repeated keys at THIS level:
    let mut changed_any = false;
    let mut new_map = serde_json::Map::new();
    for (key, value) in obj.into_iter() {
        match value {
            serde_json::Value::Object(inner) if inner.len() == 1 && inner.contains_key(&key) => {
                debug!("Flattening repeated '{}' layer to remove nesting", key);
                let nested_val = inner.get(&key).cloned().unwrap();
                new_map.insert(key, nested_val);
                changed_any = true;
            }
            other => {
                new_map.insert(key, other);
            }
        }
    }
    if changed_any {
        serde_json::Value::Object(new_map)
    } else {
        // No changes, return original shape
        serde_json::Value::Object(new_map)
    }
}
