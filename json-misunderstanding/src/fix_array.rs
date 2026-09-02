crate::ix!();

/// fix_array merges your old #2..#10, #14 transformations plus new #16..#21, #30 stubs at the end.
pub fn fix_array(
    arr: Vec<serde_json::Value>,
    config: &MisunderstandingCorrectionConfig,
) -> serde_json::Value {
    tracing::trace!("Visiting an array node for corrections");

    // Step 1: Recurse on items
    let mut corrected: Vec<serde_json::Value> = arr
        .into_iter()
        .map(|item| fix_value(item, config))
        .collect();

    // Step 2: The #2..#3..#10..#14 transformations from your old code:
    // #2 nested vector flattening
    if *config.handle_nested_vector_flattening() {
        corrected = fix_nested_vector_flattening(corrected);
    }
    // #3 single-element vector omission (naive)
    if *config.handle_single_element_vector_omission() {
        tracing::debug!("single_element_vector_omission is enabled, but no strong transform is done");
    }
    // #10 & #14 array-wrapped single objects or singleton array
    if *config.handle_array_wrapped_single_objects()
        || *config.handle_singleton_array_instead_of_object()
    {
        tracing::debug!("array_wrapped_single_objects / singleton_array_instead_of_object is enabled, naive approach");
    }
    // #15 reversed map structure
    if *config.handle_reversed_map_structure() && !corrected.is_empty() {
        let all_kv = corrected.iter().all(|v| {
            match v {
                serde_json::Value::Object(o) => o.contains_key("key") && o.contains_key("value"),
                _ => false,
            }
        });
        if all_kv {
            let mut new_map = serde_json::Map::new();
            for element in corrected {
                if let serde_json::Value::Object(o) = element {
                    if let (Some(k), Some(v)) = (o.get("key"), o.get("value")) {
                        if let Some(k_str) = k.as_str() {
                            new_map.insert(k_str.to_owned(), v.clone());
                        }
                    }
                }
            }
            tracing::debug!("Reversed map structure fixed by creating an object from key-value pairs");
            return serde_json::Value::Object(new_map);
        }
    }

    // Step 3: Add your new stubs #16, #21, #30 at the end
    if *config.handle_mixed_type_arrays() {
        // #16
        corrected = match fix_mixed_type_arrays(serde_json::Value::Array(corrected)) {
            serde_json::Value::Array(a) => a,
            other => vec![other],
        };
    }
    if *config.handle_deeply_nested_vector_overwrap() {
        // #21
        corrected = match fix_deeply_nested_vector_overwrap(serde_json::Value::Array(corrected)) {
            serde_json::Value::Array(a) => a,
            other => vec![other],
        };
    }
    if *config.handle_scalar_to_array_repetition() {
        // #30
        corrected = match fix_scalar_to_array_repetition(serde_json::Value::Array(corrected)) {
            serde_json::Value::Array(a) => a,
            other => vec![other],
        };
    }

    serde_json::Value::Array(corrected)
}


