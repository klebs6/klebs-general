crate::ix!();

pub fn repair_json_fix_mismatched_quotes(input: &str) -> String {
    let mut repaired = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut quote_char = '\0';

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escaped {
            // Previous character was a backslash; current character is escaped
            repaired.push(c);
            escaped = false;
        } else if c == '\\' {
            // Start of an escape sequence
            repaired.push(c);
            escaped = true;
        } else if c == '"' || c == '\'' {
            if in_string {
                if c == quote_char {
                    // Potential end of string
                    if let Some(&next_c) = chars.peek() {
                        if next_c.is_alphanumeric() || next_c == '\\' {
                            // Likely a quote inside the string, escape it
                            repaired.push('\\');
                            repaired.push(c);
                        } else {
                            // End of string
                            repaired.push(c);
                            in_string = false;
                            quote_char = '\0';
                        }
                    } else {
                        // End of input, end of string
                        repaired.push(c);
                        in_string = false;
                        quote_char = '\0';
                    }
                } else {
                    // Mismatched quote inside string, escape it
                    repaired.push('\\');
                    repaired.push(c);
                }
            } else {
                // Not in a string
                repaired.push(c);
                in_string = true;
                quote_char = c;
            }
        } else if in_string {
            if c == ',' || c == '}' || c == ']' {
                // Possible missing closing quote before comma or brace/bracket
                let mut temp_chars = chars.clone();
                while let Some(&nc) = temp_chars.peek() {
                    if nc.is_whitespace() {
                        temp_chars.next();
                    } else {
                        break;
                    }
                }
                if let Some(&nc) = temp_chars.peek() {
                    if nc == quote_char || nc == ':' || nc == ',' || nc == '}' || nc == ']' {
                        // Missing closing quote
                        repaired.push(quote_char);
                        in_string = false;
                        quote_char = '\0';
                    }
                } else {
                    // End of input, insert closing quote
                    repaired.push(quote_char);
                    in_string = false;
                    quote_char = '\0';
                }
                repaired.push(c);
            } else {
                repaired.push(c);
            }
        } else {
            repaired.push(c);
            // Check for missing opening quote after colon
            if c == ':' {
                // Collect whitespace
                let mut temp_chars = chars.clone();
                let mut whitespace = String::new();
                while let Some(&nc) = temp_chars.peek() {
                    if nc.is_whitespace() {
                        whitespace.push(nc);
                        temp_chars.next();
                    } else {
                        break;
                    }
                }
                if let Some(&nc) = temp_chars.peek() {
                    if nc != '"' && nc != '\'' && !is_valid_json_value_start(nc) {
                        // Insert collected whitespace
                        repaired.push_str(&whitespace);
                        // Insert missing opening quote
                        repaired.push('"');
                        in_string = true;
                        quote_char = '"';
                        // Advance the main iterator past the whitespace
                        for _ in 0..whitespace.len() {
                            chars.next();
                        }
                    }
                } else {
                    // No more characters, insert collected whitespace and opening quote
                    repaired.push_str(&whitespace);
                    repaired.push('"');
                    in_string = true;
                    quote_char = '"';
                    // Advance the main iterator past the whitespace
                    for _ in 0..whitespace.len() {
                        chars.next();
                    }
                }
            }
        }
    }

    if in_string {
        // Unclosed string at the end; add closing quote
        repaired.push(quote_char);
    }

    repaired
}

fn is_valid_json_value_start(c: char) -> bool {
    c == '"' || c == '\'' || c == '{' || c == '[' || c.is_ascii_digit() || c == '-' || c == 't' || c == 'f' || c == 'n'
}

#[cfg(test)]
mod repair_json_fix_mismatched_quotes_tests {
    use super::*;

    #[test]
    fn test_missing_closing_quote_before_comma() {
        let input = r#"{"key": "value, "another_key": "another_value"}"#;
        let expected = r#"{"key": "value", "another_key": "another_value"}"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_missing_closing_quote_at_end() {
        let input = r#"{"key": "value}"#;
        let expected = r#"{"key": "value"}"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_missing_opening_quote() {
        let input = r#"{"key": value"}"#;
        let expected = r#"{"key": "value"}"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_nested_mismatched_quotes() {
        let input = r#"{"key": "val"ue"}"#;
        let expected = r#"{"key": "val\"ue"}"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_multiple_missing_quotes_in_array() {
        let input = r#"["item1", "item2, "item3", "item4, "item5"]"#;
        let expected = r#"["item1", "item2", "item3", "item4", "item5"]"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_string_with_embedded_comma_without_closing_quote() {
        let input = r#"{"key": "value with comma, another part"}"#;
        let expected = r#"{"key": "value with comma, another part"}"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_string_with_escaped_quote() {
        let input = r#"{"key": "value with an escaped quote \" still in string"}"#;
        let expected = r#"{"key": "value with an escaped quote \" still in string"}"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_properly_quoted_string() {
        let input = r#"{"key": "value"}"#;
        let expected = r#"{"key": "value"}"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }

    #[test]
    fn test_unclosed_string_in_nested_object() {
        let input = r#"{"outer": {"inner": "value}}"#;
        let expected = r#"{"outer": {"inner": "value"}}"#;
        let output = repair_json_fix_mismatched_quotes(input);
        assert_eq!(output, expected);
    }
}
