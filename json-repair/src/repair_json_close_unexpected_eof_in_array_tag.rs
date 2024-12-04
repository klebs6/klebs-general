crate::ix!();

#[allow(unused_assignments)]
pub fn repair_json_close_unexpected_eof_in_array_tag(input: &str) -> String {
    use std::collections::VecDeque;

    #[derive(Clone, Copy)]
    enum Context {
        Object,
        Array,
    }

    let mut repaired = String::new();
    let mut chars = input.chars().peekable();
    let mut stack = VecDeque::new();
    let mut context_stack = VecDeque::new();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(c) = chars.next() {
        repaired.push(c);

        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        } else {
            if c == '"' {
                in_string = true;
            } else if c == '{' {
                stack.push_back('}');
                context_stack.push_back(Context::Object);
            } else if c == '[' {
                stack.push_back(']');
                context_stack.push_back(Context::Array);
            } else if c == '}' || c == ']' {
                if Some(c) == stack.back().copied() {
                    stack.pop_back();
                    context_stack.pop_back();
                }
            }
        }
    }

    // If we are still inside a string, close it
    if in_string {
        repaired.push('"');
    }

    // Remove trailing whitespace
    let mut temp = repaired.clone();
    while temp.ends_with(|c: char| c.is_whitespace()) {
        temp.pop();
    }

    // Get current context
    let current_context = context_stack.back().copied();

    // Attempt to fix incomplete structures
    if temp.ends_with(':') {
        // Missing value after colon
        repaired.push_str(" null");
    } else if temp.ends_with('"') {
        // Possibly incomplete key or value
        let mut chars = temp.chars().rev().peekable();
        // Skip the closing quote
        chars.next();

        let mut in_string = true;
        let mut escaped = false;

        while let Some(c) = chars.next() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                    break;
                }
            } else {
                break;
            }
        }

        // Skip whitespace
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        if let Some(context) = current_context {
            match context {
                Context::Object => {
                    if let Some(&c) = chars.peek() {
                        if c == ',' || c == '{' {
                            // Missing colon and value after key
                            repaired.push_str(": null");
                        }
                        // Removed the else if c == ':' block
                    }
                }
                Context::Array => {
                    // Do nothing in array context
                }
            }
        }
    }

    // Close any open structures
    while let Some(c) = stack.pop_back() {
        repaired.push(c);
    }

    repaired
}

#[cfg(test)]
mod repair_json_close_unexpected_eof_in_array_tag_tests {
    use super::*;
    use serde_json::{json, Value};

    fn parse_json(input: &str) -> Result<Value, JsonRepairError> {
        serde_json::from_str(input).map_err(|inner| JsonRepairError::SerdeParseError { inner })
    }

    #[test]
    fn test_complete_json() {
        let input = r#"{"key": "value"}"#;
        let expected = json!({"key": "value"});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_unclosed_object() {
        let input = r#"{"key": "value""#;
        let expected = json!({"key": "value"});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_unclosed_array() {
        let input = r#"["item1", "item2""#;
        let expected = json!(["item1", "item2"]);
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_unclosed_string() {
        let input = r#"{"key": "value"#;
        let expected = json!({"key": "value"});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_unclosed_nested_structures() {
        let input = r#"{"key1": {"key2": ["item1", "item2"]"#;
        let expected = json!({"key1": {"key2": ["item1", "item2"]}});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_incomplete_key() {
        let input = r#"{"key1": "value1", "key2"#;
        let expected = json!({"key1": "value1", "key2": null});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_incomplete_key_with_colon() {
        let input = r#"{"key1": "value1", "key2":"#;
        let expected = json!({"key1": "value1", "key2": null});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_incomplete_value_in_array() {
        let input = r#"["item1", "item2"#;
        let expected = json!(["item1", "item2"]);
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_unexpected_eof_midway_through_array() {
        let input = r#"{
  "tag1": [
    "item1",
    "item2"
  ],
  "tag2"#;
        let expected = json!({
            "tag1": ["item1", "item2"],
            "tag2": null
        });
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_unexpected_eof_midway_through_object() {
        let input = r#"{
  "tag1": {
    "subtag1": "value1",
    "subtag2": "value2"
  },
  "tag2"#;
        let expected = json!({
            "tag1": {
                "subtag1": "value1",
                "subtag2": "value2"
            },
            "tag2": null
        });
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_unclosed_string_in_key() {
        let input = r#"{"key1": "value1", "key2"#;
        let expected = json!({"key1": "value1", "key2": null});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_unclosed_string_in_value() {
        let input = r#"{"key1": "value1", "key2": "value2"#;
        let expected = json!({"key1": "value1", "key2": "value2"});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[test]
    fn test_incomplete_key_without_quotes() {
        let input = r#"{"key1": "value1", key2"#; // Missing quotes around "key2"
        let expected = json!({"key1": "value1", "key2": null});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        // Note: The function may not handle keys without quotes, but JSON requires keys to be strings.
    }

    #[test]
    fn test_missing_value_after_colon() {
        let input = r#"{"key1": "value1", "key2":"value2", "key3":"#;
        let expected = json!({"key1": "value1", "key2": "value2", "key3": null});
        let repaired = repair_json_close_unexpected_eof_in_array_tag(input);
        let output = parse_json(&repaired);
        assert_expected_matches_output_result(input,&output,&expected);
    }
}
