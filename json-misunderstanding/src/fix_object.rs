crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn fix_object(
    mut map: serde_json::Map<String, serde_json::Value>,
    config: &MisunderstandingCorrectionConfig,
) -> serde_json::Value {
    trace!("Entering fix_object with an object of size {}", map.len());

    // ------------------------------------------------------------------
    // STEP 1) Single-Key logic (#1 map_vector_confusion, #24 data unwrap,
    //         #8 results flatten) happens *before* recursing on children.
    //         This ensures that if we transform the object into an array
    //         (like for map_vector_confusion), we still get a chance to
    //         fix the children (e.g. rename "descriptor"→"description")
    //         in a subsequent pass.
    // ------------------------------------------------------------------
    if map.len() == 1 {
        let (sole_key, sole_val) = {
            let (k, v) = map.iter().next().unwrap();
            (k.clone(), v.clone())
        };

        // #24 incorrectly_nested_data_wrapper: unwrap if the single key = "data" -> array
        if *config.handle_incorrectly_nested_data_wrapper()
            && sole_key == "data"
            && sole_val.is_array()
        {
            debug!("Unwrapping single-key object 'data' -> returning that array");
            return sole_val;
        }

        // #8 unnecessary_additional_nesting: if single key = "results", and inside is
        //    an object with "items" = array, keep 'results' but swap in that array
        if *config.handle_unnecessary_additional_nesting() && sole_key == "results" {
            if let serde_json::Value::Object(inner_obj) = &sole_val {
                if let Some(serde_json::Value::Array(items_arr)) = inner_obj.get("items") {
                    debug!("Flattening 'results.items' -> keep 'results' key with array");
                    let mut out_map = serde_json::Map::new();
                    out_map.insert("results".to_owned(), serde_json::Value::Array(items_arr.clone()));
                    return serde_json::Value::Object(out_map);
                }
            }
        }

        // #1 map_vector_confusion
        //    If that triggers, we want to transform the object into an array, then
        //    continue fixing that array's children (for key_name_misalignment, etc).
        if *config.handle_map_vector_confusion() {
            let before = serde_json::Value::Object(map.clone());
            let after = fix_map_vector_confusion(before.clone());
            if after != before {
                debug!("map_vector_confusion triggered -> will finalize by re-running fix_value on the transformed data");
                // We re-run fix_value on the new structure to ensure subsequent
                // fixes like key-name misalignment are applied to the child array/object.
                return fix_value(after, config);
            }
        }
    }

    // ------------------------------------------------------------------
    // STEP 2) Recurse on children
    // ------------------------------------------------------------------
    for (k, v) in map.iter_mut() {
        *v = fix_value(std::mem::take(v), config);
    }

    // ------------------------------------------------------------------
    // STEP 3) Apply object-level transforms (#4, #15, #19..#29, #11)
    // ------------------------------------------------------------------
    let mut out = serde_json::Value::Object(map);

    if *config.handle_vector_as_map_of_indices() {
        let before = out.clone();
        let after = fix_vector_as_map_of_indices(before.clone());
        if after != before {
            debug!("Applied fix_vector_as_map_of_indices");
            out = after;
        }
    }

    if *config.handle_reversed_map_structure() {
        let before = out.clone();
        let after = fix_reversed_map_structure(before.clone());
        if after != before {
            debug!("Applied fix_reversed_map_structure");
            out = after;
        }
    }

    if *config.handle_misplaced_metadata() {
        out = fix_misplaced_metadata(out);
    }
    if *config.handle_enumeration_as_map() {
        out = fix_enumeration_as_map(out);
    }
    if *config.handle_overly_verbose_field() {
        out = fix_overly_verbose_field(out);
    }
    if *config.handle_numeric_keys_misunderstanding() {
        out = fix_numeric_keys_misunderstanding(out);
    }
    if *config.handle_redundant_nesting_of_identical_keys() {
        out = fix_redundant_nesting_of_identical_keys(out);
    }
    if *config.handle_flattened_pairs() {
        out = fix_flattened_pairs(out);
    }

    // #11 key_name_misalignment
    if *config.handle_key_name_misalignment() {
        out = fix_key_name_misalignment(out);
    }

    trace!("Leaving fix_object with possibly-transformed object node.");
    out
}
