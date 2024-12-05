crate::ix!();

pub fn repair_json_accidental_single_quote_instead_of_double_quote(
    input: &str,
) -> Result<String, JsonRepairError> {
    let (output, made_correction) = fix_mismatched_quotes(input);
    if made_correction {
        info!("Fixed accidental single quotes in JSON strings.");
    }
    Ok(output)
}

fn fix_mismatched_quotes(input: &str) -> (String, bool) {
    use std::iter::Peekable;
    use std::str::Chars;

    let mut output = String::new();
    let mut chars: Peekable<Chars> = input.chars().peekable();
    let mut inside_string = false;
    let mut string_delimiter = '\0';
    let mut escape = false;
    let mut made_correction = false;

    while let Some(c) = chars.next() {
        output.push(c);

        if escape {
            // Current character is escaped; reset escape flag.
            escape = false;
            continue;
        }

        if c == '\\' {
            // Next character is escaped.
            escape = true;
            continue;
        }

        if c == '"' || c == '\'' {
            if !inside_string {
                // Starting a string.
                inside_string = true;
                string_delimiter = c;
            } else if c == string_delimiter {
                // Ending the string.
                inside_string = false;
            } else {
                // Mismatched quote inside string.
                // Peek ahead to see if it's an accidental closing quote.
                let mut peek_iter = chars.clone();
                while let Some(&next_c) = peek_iter.peek() {
                    if next_c.is_whitespace() {
                        peek_iter.next();
                    } else {
                        break;
                    }
                }

                if let Some(&next_c) = peek_iter.peek() {
                    if next_c == ',' || next_c == ']' || next_c == '}' {
                        // Likely an accidental closing quote.
                        // Replace the mismatched quote with the correct one in the output.
                        output.pop(); // Remove the mismatched quote.
                        output.push(string_delimiter); // Replace with correct delimiter.
                        inside_string = false;
                        made_correction = true;
                    } else {
                        // It's a quote character inside the string value; escape it.
                        output.insert(output.len() - 1, '\\'); // Insert backslash before quote.
                        made_correction = true;
                    }
                } else {
                    // End of input; treat as closing quote.
                    output.pop(); // Remove the mismatched quote.
                    output.push(string_delimiter); // Replace with correct delimiter.
                    inside_string = false;
                    made_correction = true;
                }
            }
        }
    }

    (output, made_correction)
}

#[cfg(test)]
mod repair_accidental_single_quote_tests {
    use super::*;
    use serde_json::json;
    use serde_json::Value;

    #[test]
    fn test_repair_single_quote_instead_of_double_quote() -> Result<(), JsonRepairError> {
        // value4 has a single quote instead of a double
        let input = r#"{
            "key": [
                "value1",
                "value2",
                "value3",
                "value4',
                "value5",
                "value6"
            ]
        }"#;

        let expected = json!({
            "key": [
                "value1",
                "value2",
                "value3",
                "value4",
                "value5",
                "value6"
            ]
        });

        let output = repair_json_accidental_single_quote_instead_of_double_quote(input)?;

        // Parse output as JSON
        let output_json: Value = serde_json::from_str(&output)
            .map_err(|inner| JsonRepairError::SerdeParseError { inner })?;

        assert_eq!(output_json, expected);

        Ok(())
    }
}
