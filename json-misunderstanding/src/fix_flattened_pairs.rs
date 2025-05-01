crate::ix!();

/// #29 Fixes "flattened objects into arrays of pairs" e.g.
/// {"settings": [["volume",80],["brightness",60]]} -> {"settings": {"volume":80,"brightness":60}}
/// We'll do a naive approach: for any array of arrays with length 2, convert to an object.
pub fn fix_flattened_pairs(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_flattened_pairs");
    match val {
        serde_json::Value::Object(mut obj) => {
            // For each field that is an array, check if it's an array of [key, value]
            for (k, v) in obj.iter_mut() {
                if let serde_json::Value::Array(arr) = v {
                    let can_convert = arr.iter().all(|x| {
                        x.as_array().map_or(false, |a| a.len() == 2)
                    });
                    if can_convert {
                        debug!("Converting array-of-pairs to object in field '{}'", k);
                        let mut new_map = serde_json::Map::new();
                        for pair in arr.iter() {
                            if let Some(pair_arr) = pair.as_array() {
                                let key_val = &pair_arr[0];
                                let val_val = &pair_arr[1];
                                // For simplicity, if key_val is string-like, use it
                                if let Some(k_str) = key_val.as_str() {
                                    new_map.insert(k_str.to_owned(), val_val.clone());
                                }
                            }
                        }
                        *v = serde_json::Value::Object(new_map);
                    }
                }
            }
            serde_json::Value::Object(obj)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_flattened_pairs {
    use super::*;

    #[traced_test]
    fn test_array_of_pairs_to_object() {
        trace!("Testing fix_flattened_pairs with array-of-pairs data");
        let input = json!({
            "settings": [
                ["volume", 80],
                ["brightness", 60]
            ],
            "other": {"unaffected": true}
        });
        debug!("Input: {}", input);

        let expected = json!({
            "settings": {"volume":80,"brightness":60},
            "other": {"unaffected": true}
        });
        let output = fix_flattened_pairs(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Array-of-pairs should be converted to object");
        info!("Flattened array-of-pairs into object successfully");
    }

    #[traced_test]
    fn test_non_pair_arrays() {
        trace!("Testing fix_flattened_pairs with arrays that are not pairs");
        let input = json!({
            "settings": [
                ["volume", 80, "extra"], 
                ["brightness"]
            ]
        });
        debug!("Input: {}", input);

        let output = fix_flattened_pairs(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "No conversion when sub-arrays aren't exactly [key, value]");
        info!("No changes for arrays not matching 2-element pairs pattern");
    }
}
