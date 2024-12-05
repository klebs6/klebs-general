crate::ix!();

pub fn remove_control_characters_in_value(value: &mut Value) -> bool {
    let mut changed = false;
    match value {
        Value::String(s) => {
            let original = s.clone();
            let cleaned: String = s
                .chars()
                .filter(|&c| (c >= '\u{20}' && c <= '\u{10FFFF}') || c == '\n')
                .collect();

            if cleaned != *s {
                *s = cleaned;
                changed = true;
            }
        },
        Value::Array(arr) => {
            for v in arr {
                if remove_control_characters_in_value(v) {
                    changed = true;
                }
            }
        },
        Value::Object(map) => {
            for v in map.values_mut() {
                if remove_control_characters_in_value(v) {
                    changed = true;
                }
            }
        },
        _ => {}
    }

    if changed {
        info!("Removed control characters from JSON value.");
    }
    changed
}

#[cfg(test)]
mod remove_control_characters_in_value_tests {
    use super::*;
    use serde_json::json;

    #[traced_test]
    fn test_string_with_control_characters() {
        let mut value = json!("This is a test\u{0001}string with control characters");
        remove_control_characters_in_value(&mut value);
        assert_eq!(value, json!("This is a teststring with control characters"));
    }

    #[traced_test]
    fn test_string_without_control_characters() {
        let mut value = json!("This is a normal string");
        remove_control_characters_in_value(&mut value);
        assert_eq!(value, json!("This is a normal string"));
    }

    #[traced_test]
    fn test_nested_objects() {
        let mut value = json!({
            "text": "Some text\u{0002}",
            "number": 42,
            "array": ["item1", "item\u{0003}2"],
            "object": {
                "nested_text": "\u{0004}Nested text",
                "bool": true
            }
        });
        remove_control_characters_in_value(&mut value);
        let expected = json!({
            "text": "Some text",
            "number": 42,
            "array": ["item1", "item2"],
            "object": {
                "nested_text": "Nested text",
                "bool": true
            }
        });
        assert_eq!(value, expected);
    }

    #[traced_test]
    fn test_array_of_values() {
        let mut value = json!(["String\u{0005} with control char", 123, true, null, "\u{0006}Another string"]);
        remove_control_characters_in_value(&mut value);
        let expected = json!(["String with control char", 123, true, null, "Another string"]);
        assert_eq!(value, expected);
    }

    #[traced_test]
    fn test_empty_string() {
        let mut value = json!("");
        remove_control_characters_in_value(&mut value);
        assert_eq!(value, json!(""));
    }

    #[traced_test]
    fn test_string_with_only_control_characters() {
        let mut value = json!("\u{0007}\u{0008}\u{0009}");
        println!("input: {:#?}", value);
        remove_control_characters_in_value(&mut value);
        let expected = json!("");
        println!("output: {:#?}", value);
        println!("expected: {:#?}", expected);
        assert_eq!(value, expected);
    }

    #[traced_test]
    fn test_no_strings_in_json() {
        let mut value = json!({
            "number": 123,
            "boolean": false,
            "null_value": null,
            "array": [1, 2, 3]
        });
        let expected = value.clone();
        remove_control_characters_in_value(&mut value);
        assert_eq!(value, expected);
    }

    #[traced_test]
    fn test_control_characters_in_keys() {
        let mut value = json!({
            "key\u{000A}1": "value1",
            "key2": "value\u{000B}2",
            "\u{000C}key3": "value3"
        });
        remove_control_characters_in_value(&mut value);
        // Note: Control characters in keys are not removed by this function
        let expected = json!({
            "key\u{000A}1": "value1",
            "key2": "value2",
            "\u{000C}key3": "value3"
        });
        assert_eq!(value, expected);
    }

    #[traced_test]
    fn test_unicode_characters() {
        let mut value = json!("Unicode test: \u{1F600}\u{000D}");
        remove_control_characters_in_value(&mut value);
        assert_eq!(value, json!("Unicode test: \u{1F600}"));
    }

    #[traced_test]
    fn test_nested_arrays() {
        let mut value = json!([
            "string\u{000E}",
            ["nested\u{000F}string", 123],
            {"key": "value\u{0010}"}
        ]);
        remove_control_characters_in_value(&mut value);
        let expected = json!([
            "string",
            ["nestedstring", 123],
            {"key": "value"}
        ]);
        assert_eq!(value, expected);
    }
}
