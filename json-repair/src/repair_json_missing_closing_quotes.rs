crate::ix!();

pub fn repair_json_missing_closing_quotes(input: &str) -> Result<String, JsonRepairError> {
    let mut output        = String::with_capacity(input.len());
    let mut chars         = input.chars().peekable();
    let mut inside_string = false;
    let mut escape        = false;
    let mut changed       = false;
    let mut ended_with_backslash = false;
    let mut treat_next_quote_as_literal = false; // New flag

    // Stack to track braces/brackets
    let mut stack = Vec::new();

    // Allows reprocessing a character when a string closes prematurely
    let mut reprocess_char: Option<char> = None;

    loop {
        let c = if let Some(rc) = reprocess_char.take() {
            rc
        } else {
            match chars.next() {
                Some(ch) => ch,
                None => break,
            }
        };

        if escape {
            output.push(c);
            escape = false;
            ended_with_backslash = c == '\\' && inside_string;
            continue;
        }

        if c == '\\' && inside_string {
            output.push(c);
            escape = true;
            ended_with_backslash = true;
            continue;
        }

        if inside_string {
            match c {
                '"' => {
                    // Properly close the string
                    output.push(c);
                    inside_string = false;
                    ended_with_backslash = false;
                }
                ',' | ']' | '}' => {
                    // Close the string before these delimiters
                    output.push('"');
                    inside_string = false;
                    changed = true;
                    reprocess_char = Some(c);
                    ended_with_backslash = false;
                    continue;
                }
                '\n' | '\r' => {
                    // Close the string before newline
                    output.push('"');
                    inside_string = false;
                    changed = true;
                    ended_with_backslash = false;
                    output.push(c); 
                    // After handling newline, we want to allow the next quote to be literal if it appears.
                    treat_next_quote_as_literal = true;
                    continue;
                }
                _ => {
                    output.push(c);
                    ended_with_backslash = false;
                }
            }
        } else {
            // Outside a string
            if treat_next_quote_as_literal && c == '"' {
                // Output the quote as literal character and reset the flag
                output.push(c);
                treat_next_quote_as_literal = false;
                continue;
            }

            match c {
                '"' => {
                    inside_string = true;
                    output.push(c);
                    ended_with_backslash = false;
                }
                '{' => {
                    stack.push('}');
                    output.push(c);
                }
                '[' => {
                    stack.push(']');
                    output.push(c);
                }
                '}' | ']' => {
                    if let Some(expected) = stack.pop() {
                        if expected != c {
                            // Mismatch: insert the expected closing and skip this one
                            output.push(expected);
                            changed = true;
                            continue;
                        } else {
                            output.push(c);
                        }
                    } else {
                        // Extra closing bracket: skip it
                        changed = true;
                        continue;
                    }
                }
                _ => {
                    output.push(c);
                }
            }
        }
    }

    // If we ended inside a string, close it properly.
    if inside_string {
        changed = true;
        if ended_with_backslash {
            // Remove the trailing backslash
            if let Some(last_char) = output.pop() {
                if last_char != '\\' {
                    // Restore if we didn't actually remove a backslash
                    output.push(last_char);
                }
            }
            // Append the sequence that represents a closed string after a trailing backslash.
            output.push_str("\\\"\\\"");
        } else {
            output.push('"');
        }
    }

    // Close any remaining braces/brackets
    if !stack.is_empty() {
        changed = true;
        while let Some(expected) = stack.pop() {
            output.push(expected);
        }
    }

    // If no changes were made, return the original input
    if !changed {
        return Ok(input.to_string());
    }

    // Remove spaces after commas
    let output = output.replace(", ", ",");

    info!("Repaired missing closing quotes.");
    Ok(output)
}

#[cfg(test)]
mod tests {
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
        let input = r#"{"key": "value", "another_key": "another value"}"#;
        let expected = r#"{"key": "value", "another_key": "another value"}"#;
        let output = repair_json_missing_closing_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_missing_closing_quote_before_bracket() -> Result<(), JsonRepairError> {
        init_tracing();
        let input = r#"{"list": ["item1, "item2"]"#;
        let expected = r#"{"list": ["item1","item2"]}"#;
        let output = repair_json_missing_closing_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_missing_closing_quote_before_newline() -> Result<(), JsonRepairError> {
        init_tracing();
        let input    = "{\n\"key\": \"value\n\"}";
        let expected = "{\n\"key\": \"value\"\n\"}";
        let output = repair_json_missing_closing_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_escaped_quotes_in_string() -> Result<(), JsonRepairError> {
        init_tracing();
        let input = r#"{"key": "value with \"escaped\" quote, "another": "thing"#;
        let expected = r#"{"key": "value with \"escaped\" quote","another": "thing"}"#;
        let output = repair_json_missing_closing_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_missing_closing_quote_at_end_of_input() -> Result<(), JsonRepairError> {
        init_tracing();
        let input = r#"{"key": "value"#;
        let expected = r#"{"key": "value"}"#;
        let output = repair_json_missing_closing_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_missing_closing_quote_before_brace() -> Result<(), JsonRepairError> {
        init_tracing();
        let input = r#"{"object": {"key": "value, "another": "something"}"#;
        let expected = r#"{"object": {"key": "value","another": "something"}}"#;
        let output = repair_json_missing_closing_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_string_with_escape_at_end() -> Result<(), JsonRepairError> {
        init_tracing();
        let input = r#"{"key": "value with a backslash \"#;
        let expected = r#"{"key": "value with a backslash \"\"}"#;
        let output = repair_json_missing_closing_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_multiple_missing_closing_quotes() -> Result<(), JsonRepairError> {
        init_tracing();
        let input = r#"{"key1": "val1, "key2": "val2, "key3": "val3"#;
        let expected = r#"{"key1": "val1","key2": "val2","key3": "val3"}"#;
        let output = repair_json_missing_closing_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }
}

