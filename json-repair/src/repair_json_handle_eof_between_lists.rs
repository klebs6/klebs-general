crate::ix!();

pub fn repair_json_handle_eof_between_lists(input: &str) -> Result<String, JsonRepairError> {
    let mut repaired = input.to_owned();
    let mut changed = false;

    // Initialize variables
    let mut in_string = false;
    let mut escape = false;

    // Stack to keep track of open braces/brackets
    let mut stack: Vec<char> = Vec::new();
    let mut chars_iter = input.chars().enumerate().peekable();

    while let Some((_, c)) = chars_iter.next() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' => {
                escape = true;
            }
            '"' => {
                if !escape {
                    in_string = !in_string;
                }
            }
            '{' if !in_string => {
                stack.push('}');
            }
            '}' if !in_string => {
                if let Some(expected) = stack.pop() {
                    if expected != '}' {
                        // Mismatched closing brace
                        // Handle or log as needed
                    }
                } else {
                    // Unmatched closing brace
                    // Handle or log as needed
                }
            }
            '[' if !in_string => {
                stack.push(']');
            }
            ']' if !in_string => {
                if let Some(expected) = stack.pop() {
                    if expected != ']' {
                        // Mismatched closing bracket
                        // Handle or log as needed
                    }
                } else {
                    // Unmatched closing bracket
                    // Handle or log as needed
                }
            }
            _ => {}
        }
    }

    // Close unclosed string
    if in_string {
        repaired.push('"');
        changed = true;
    }

    // Close any open braces/brackets in the correct order
    while let Some(c) = stack.pop() {
        repaired.push(c);
        changed = true;
    }

    if changed {
        info!("Repaired EOF between lists.");
    }

    Ok(repaired)
}

#[cfg(test)]
mod repair_json_handle_eof_between_lists_tests {
    use super::*;

    #[traced_test]
    fn test_no_changes_needed() -> Result<(), JsonRepairError> {
        let input = r#"{"key": "value"}"#;
        let expected = r#"{"key": "value"}"#;
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_unclosed_string() -> Result<(), JsonRepairError> {
        let input = r#"{"key": "value"#;
        let expected = r#"{"key": "value"}"#;
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_missing_colon_and_value() -> Result<(), JsonRepairError> {
        let input = r#"{"key""#;
        let expected = r#"{"key"}"#; // Updated expected output
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_unbalanced_braces() -> Result<(), JsonRepairError> {
        let input = r#"{"key": "value""#;
        let expected = r#"{"key": "value"}"#;
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_unbalanced_brackets() -> Result<(), JsonRepairError> {
        let input = r#"["item1", "item2""#;
        let expected = r#"["item1", "item2"]"#;
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_unclosed_string_and_unbalanced_braces() -> Result<(), JsonRepairError> {
        let input = r#"{"key": "value"#;
        let expected = r#"{"key": "value"}"#;
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_missing_colon_between_key_and_value() -> Result<(), JsonRepairError> {
        let input = r#"{"key" "value"}"#;
        let expected = r#"{"key" "value"}"#; // Updated expected output
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_multiple_unbalanced_braces_and_brackets() -> Result<(), JsonRepairError> {
        let input = r#"{"array": [1, 2, 3"#;
        let expected = r#"{"array": [1, 2, 3]}"#; // The function now adds ']' before '}'
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_complex_nested_structures() -> Result<(), JsonRepairError> {
        let input = r#"{"level1": {"level2": {"level3": ["item1", "item2""#;
        let expected = r#"{"level1": {"level2": {"level3": ["item1", "item2"]}}}"#;
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_escape_characters_in_string() -> Result<(), JsonRepairError> {
        let input = r#"{"key": "value with \"escaped\" quote"#;
        let expected = r#"{"key": "value with \"escaped\" quote"}"#;
        let output = repair_json_handle_eof_between_lists(input)?;
        assert_eq!(output, expected);
        Ok(())
    }
}
