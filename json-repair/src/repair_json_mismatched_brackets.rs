crate::ix!();

/// Repairs mismatched or unbalanced brackets and braces in a JSON-like input.
/// - Balances braces `{}` and brackets `[]`.
/// - Skips extra closing brackets if they cannot be matched.
/// - Replaces mismatched closing brackets with the correct corresponding bracket if the top of the stack doesn't match.
/// - Does not attempt to insert missing colons or interpret keys/values; it only ensures that structural brackets are balanced.
/// - String literals are respected, and characters inside strings are not treated as structural characters.
/// - Escaped quotes inside strings are handled correctly.
pub fn repair_json_mismatched_brackets(input: &str) -> Result<String, JsonRepairError> {
    let mut repaired    = String::new();
    let mut stack       = Vec::new();
    let mut in_string   = false;
    let mut escape_next = false;
    let mut chars       = input.chars().peekable();
    let mut changed     = false;

    while let Some(c) = chars.next() {
        if c == '"' && !escape_next {
            in_string = !in_string;
        }

        if !in_string {
            match c {
                '{' | '[' => {
                    // Opening bracket/brace.
                    stack.push(c);
                }
                '}' | ']' => {
                    // Closing bracket/brace.
                    if let Some(open) = stack.pop() {
                        // Check if mismatched
                        if (open == '{' && c == ']') || (open == '[' && c == '}') {
                            // Replace mismatched closing bracket with the correct one.
                            repaired.push(if open == '{' { '}' } else { ']' });
                            changed = true;
                            continue;
                        }
                    } else {
                        // Extra closing bracket with no matching opening bracket.
                        // Skip it entirely.
                        changed = true;
                        continue;
                    }
                }
                _ => {}
            }
        } else {
            // We are inside a string
            if c == '\\' && !escape_next {
                escape_next = true;
            } else {
                escape_next = false;
            }
        }

        repaired.push(c);
    }

    // Close any unclosed brackets/braces.
    while let Some(open) = stack.pop() {
        repaired.push(if open == '{' { '}' } else { ']' });
        changed = true;
    }

    if changed {
        info!("Repaired mismatched or unbalanced brackets.");
    }

    Ok(repaired)
}

#[cfg(test)]
mod repair_json_mismatched_brackets_tests {
    use super::*;
    use tracing::{subscriber::set_global_default};
    use tracing_subscriber::{fmt, EnvFilter};

    fn init_tracing() {
        let _ = set_global_default(
            fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .finish(),
        );
    }

    #[test]
    fn test_no_changes_needed() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#"{"key": ["value1", "value2"]}"#;
        let expected = r#"{"key": ["value1", "value2"]}"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        // No changes, so no info logs expected (you can verify by checking logs)
        Ok(())
    }

    #[test]
    fn test_simple_mismatched() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#"{"array": [1, 2, 3}"#;
        // Here, the last character should be ']' to match '[', but '}' was provided.
        let expected = r#"{"array": [1, 2, 3]}"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        // Changes were made, so info logs expected.
        Ok(())
    }

    #[test]
    fn test_extra_closing_bracket() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#"{"key": "value"}}}"#;
        // Extra '}}' at the end should be skipped.
        let expected = r#"{"key": "value"}"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_unclosed_brackets() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#"{"array": [1, 2, 3"#;
        // Missing ']' and '}' at the end.
        let expected = r#"{"array": [1, 2, 3]}"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_nested_structures() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#"{"level1": {"level2": [1, 2, 3}"#;
        // The ']' should be a ']' not '}' after the array. Also, missing '}' at the end for "level2" and "level1".
        // The function will:
        // - Convert the '}' that tries to close '[1,2,3' into a ']' to properly close the array.
        // - Add '}' to close level2
        // - Add '}' to close level1
        let expected = r#"{"level1": {"level2": [1, 2, 3]}}"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_in_string_ignored() -> Result<(), JsonRepairError> {
        init_tracing();
        // Braces inside strings should be ignored.
        let input    = r#"{"key": "a string with { and } and [ and ] inside"}"#;
        let expected = r#"{"key": "a string with { and } and [ and ] inside"}"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_escaped_quotes_in_string() -> Result<(), JsonRepairError> {
        init_tracing();
        let input = r#"{"key": "value with \"escaped\" quotes [ and } inside"}"#;
        // Despite having [ and } inside the string, they are ignored.
        let expected = r#"{"key": "value with \"escaped\" quotes [ and } inside"}"#;
        let output = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_empty_input() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#""#; // empty
        let expected = r#""#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_only_string() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#""just a string""#;
        let expected = r#""just a string""#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_only_brackets_unclosed() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#"["#;
        // Missing a closing bracket
        let expected = r#"[]"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_only_braces_unclosed() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#"{"#;
        // Missing a closing brace
        let expected = r#"{}"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_mismatched_stack() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = r#"["key": {"another_array": [1,2}]"#;
        // Analysis:
        // '[' opens
        // Next char '{' is inside? Actually the string is `["key": { ... }`, meaning at some point we have:
        // '[' opened, expecting a matching ']'
        // Next '{' opened expecting a matching '}'
        // We get a '}' where a ']' might be expected. The code should fix that:
        // Actually, in this example: '[' -> stack: [']']
        // Then '{' -> stack: [']', '}']
        // Then '}' encountered. Pop stack -> expecting '}', got '}', good. stack now [']'].
        // Now we reach the end and we had a '[' at the start, so we add a ']'.
        // The input also includes `["key": ...` which might not be valid JSON, but we only fix structural brackets.
        // The corrected form would be `["key": {"another_array": [1,2]}]`
        let expected = r#"["key": {"another_array": [1,2]}]"#;
        let output   = repair_json_mismatched_brackets(input)?;
        assert_eq!(output, expected);
        Ok(())
    }
}

