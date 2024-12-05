crate::ix!();

pub fn repair_json_truncated_boolean_behavior(input: &str) -> Result<String,JsonRepairError> {

    info!("repairing any truncated booleans");

    let mut output        = String::new();
    let mut chars         = input.chars().peekable();
    let mut inside_string = false;
    let mut last_token    = String::new();
    let mut open_braces   = 0;
    let mut open_brackets = 0;

    while let Some(c) = chars.next() {
        output.push(c);

        if c == '"' {
            // Check if the quote is escaped
            let mut backslash_count = 0;

            // Start from the character before the quote, if any
            if output.len() >= 2 {
                let mut idx = output.len() - 2; // position before the quote

                loop {
                    if output.as_bytes()[idx] == b'\\' {
                        backslash_count += 1;
                        if idx == 0 {
                            break;
                        } else {
                            idx -= 1;
                        }
                    } else {
                        break;
                    }
                }
            }

            if backslash_count % 2 == 0 {
                // Even number of backslashes means the quote is not escaped
                inside_string = !inside_string;
            }
            last_token.clear();
        } else if !inside_string {
            match c {
                '{' => open_braces += 1,
                '}' => {
                    if open_braces > 0 {
                        open_braces -= 1;
                    }
                }
                '[' => open_brackets += 1,
                ']' => {
                    if open_brackets > 0 {
                        open_brackets -= 1;
                    }
                }
                _ => {}
            }

            if c.is_alphabetic() {
                last_token.push(c);
            } else {
                last_token.clear();
            }
        }
    }

    // At the end, check if last_token is an incomplete boolean and complete it
    if !inside_string {
        if last_token == "t" || last_token == "tr" || last_token == "tru" {
            let remaining = &"true"[last_token.len()..];
            output.push_str(remaining);
        } else if last_token == "f" || last_token == "fa" || last_token == "fal" || last_token == "fals" {
            let remaining = &"false"[last_token.len()..];
            output.push_str(remaining);
        }
    }

    // Close any unclosed strings
    if inside_string {
        output.push('"');
    }

    // Close any unclosed brackets and braces
    for _ in 0..open_brackets {
        output.push(']');
    }
    for _ in 0..open_braces {
        output.push('}');
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn assert_expected_matches_output_result(input: &str, output: &str, expected: &serde_json::Value) {
        let parsed_output: serde_json::Value = serde_json::from_str(output).expect("Failed to parse output JSON");
        assert_eq!(&parsed_output, expected, "Parsed output does not match expected value");
    }

    #[traced_test]
    fn test_truncated_boolean_true() -> Result<(),JsonRepairError> {
        let input = r#"{"bool": tr"#;
        let output = repair_json_truncated_boolean_behavior(input)?;
        let expected = json!({"bool": true});
        assert_expected_matches_output_result(input, &output, &expected);
        Ok(())
    }

    #[traced_test]
    fn test_truncated_boolean_false() -> Result<(),JsonRepairError> {
        let input = r#"{"bool": fal"#;
        let output = repair_json_truncated_boolean_behavior(input)?;
        let expected = json!({"bool": false});
        assert_expected_matches_output_result(input, &output, &expected);
        Ok(())
    }
}
