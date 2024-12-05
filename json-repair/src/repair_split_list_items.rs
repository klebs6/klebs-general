crate::ix!();

/// Repairs arrays of strings in a JSON value by merging items split across multiple lines.
/// A new item starts with an uppercase first character. Subsequent lines that start lowercase
/// are appended to the current item until another uppercase line is found.
pub fn repair_standard_list_items_with_possible_splits(value: Value) -> Value {
    let (result, merged) = repair_split_list_items_with_merge(value);

    if merged {
        info!("Merged split list items during processing.");
    }

    result
}

/// Internal helper to process JSON values and track whether a merge occurred.
fn repair_split_list_items_with_merge(value: Value) -> (Value, bool) {
    match value {
        Value::Array(arr) => {
            // Check if this array is purely strings
            if arr.iter().all(|v| matches!(v, Value::String(_))) {
                let strings: Vec<String> = arr.into_iter().map(|v| v.as_str().unwrap().to_string()).collect();
                let (merged_strings, merged) = merge_items(strings);
                let merged_array = merged_strings.into_iter().map(Value::String).collect();
                (Value::Array(merged_array), merged)
            } else {
                // Recursively process elements
                let mut any_merged = false;
                let processed_array: Vec<Value> = arr
                    .into_iter()
                    .map(|v| {
                        let (processed, merged) = repair_split_list_items_with_merge(v);
                        any_merged |= merged;
                        processed
                    })
                    .collect();
                (Value::Array(processed_array), any_merged)
            }
        }
        Value::Object(map) => {
            // Recursively process object fields
            let mut any_merged = false;
            let mut new_map = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                let (processed, merged) = repair_split_list_items_with_merge(v);
                any_merged |= merged;
                new_map.insert(k, processed);
            }
            (Value::Object(new_map), any_merged)
        }
        other => (other, false),
    }
}

/// Merge items in a vector of strings based on the described capitalization pattern.
/// 
/// Pattern:
/// - A new item begins when we encounter a line starting with uppercase.
/// - Subsequent lines that start lowercase are appended to the current item.
/// - The next uppercase line starts a new item.
#[allow(unused_assignments)]
fn merge_items(lines: Vec<String>) -> (Vec<String>, bool) {
    let mut merged = Vec::new();
    let mut current_item = String::new();
    let mut any_merged = false;

    for line in lines {
        if !line.trim().is_empty() && line_is_uppercase_start(&line) {
            if !current_item.trim().is_empty() {
                merged.push(current_item.trim().to_string());
                any_merged = true; // Merge detected
            }
            current_item = line; // Start new item
        } else {
            // Merge continuation lines into the current item
            if !current_item.is_empty() {
                current_item.push(' ');
                current_item.push_str(&line.trim());
                any_merged = true; // Merge detected
            } else {
                // Edge case: stray continuation line without a previous item
                current_item = line;
            }
        }
    }

    if !current_item.trim().is_empty() {
        merged.push(current_item.trim().to_string());
    }

    (merged, any_merged)
}

/// Checks if a string starts with an uppercase character.
/// Returns false if the string is empty or does not start with uppercase.
fn line_is_uppercase_start(s: &str) -> bool {
    s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_items_no_change() {
        let items = vec![
            "Monochrome palettes capturing storm-laden skies".to_string(),
            "Kinetic compositions reflecting dynamic energy".to_string(),
        ];
        let (merged, any_merged) = merge_items(items.clone());
        assert_eq!(merged, items);
    }

    #[test]
    fn test_merge_items_with_split() {
        let items = vec![
            "Transformational power and change".to_string(),
            "Nature s unpredictable force".to_string(),
            "Cleansing and renewal".to_string(),
            "Destruction and creation interplay".to_string(),
            "The Great Equalizer".to_string(),
            "Cycles of upheaval and calm".to_string(),
            "Chaos bringing order".to_string(),
            "Awakening primal emotion".to_string(),
            "Overcoming adversity".to_string(),
            "Unleashing suppressed energies".to_string(),
            "Divine wrath and benevolence".to_string(),
            "Nature s raw".to_string(),    // Lowercase start => append to previous?
            "unbridled energy".to_string(),// continues previous item
            "Signifies endings and new beginnings".to_string(),
            "Elemental convergence".to_string(),
        ];

        // Expect "Nature s raw" and "unbridled energy" to merge into one line in place of just "Nature s raw"
        let (merged, any_merged) = merge_items(items.clone());
        // Check merged array
        // Original line "Nature s raw" (uppercase start) 
        // next line "unbridled energy" (lowercase start) appended to it
        let idx = merged.iter().position(|x| x.starts_with("Nature s raw")).unwrap();
        assert!(merged[idx].contains("unbridled energy"));
        assert!(!merged.contains(&"unbridled energy".to_string()));
    }

    #[test]
    fn test_repair_in_value() {
        let input = serde_json::json!({
            "array": [
                "First item",
                "second line of first item",
                "another line continuing first item",
                "Second Item",
                "lowercase continuation",
                "THIRD ITEM"
            ],
            "nested": {
                "arr2": [
                    "CapitalStart",
                    "lowercase part",
                    "AnotherCapitalStart",
                    "yet another line"
                ]
            }
        });

        let repaired = repair_standard_list_items_with_possible_splits(input);

        // Check top-level array
        let arr = repaired.get("array").unwrap().as_array().unwrap();
        // Should have merged lines for first item
        assert_eq!(arr.len(), 3, "Should have merged array items");
        assert!(arr[0].as_str().unwrap().starts_with("First item"));
        assert!(arr[0].as_str().unwrap().contains("second line of first item"));
        assert!(arr[0].as_str().unwrap().contains("another line continuing first item"));
        assert!(arr[1].as_str().unwrap().starts_with("Second Item"));
        assert!(arr[1].as_str().unwrap().contains("lowercase continuation"));
        assert_eq!(arr[2].as_str().unwrap(), "THIRD ITEM");

        // Check nested array
        let arr2 = repaired.pointer("/nested/arr2").unwrap().as_array().unwrap();
        assert_eq!(arr2.len(), 2);
        assert!(arr2[0].as_str().unwrap().contains("CapitalStart"));
        assert!(arr2[0].as_str().unwrap().contains("lowercase part"));
        assert!(arr2[1].as_str().unwrap().contains("AnotherCapitalStart"));
        assert!(arr2[1].as_str().unwrap().contains("yet another line"));
    }
}
