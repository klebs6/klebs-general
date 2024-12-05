crate::ix!();

pub fn repair_json_comma_behavior(input: &str) -> Result<String, JsonRepairError> {
    let mut output          = String::with_capacity(input.len());
    let mut chars           = input.chars().peekable();
    let mut inside_string   = false;
    let mut prev_char       = None;
    let mut string_content  = String::new();
    let mut changed         = false;
    let mut changes_made    = Vec::new();

    while let Some(c) = chars.next() {
        if c == '"' && prev_char != Some('\\') {
            // Toggle inside_string state
            inside_string = !inside_string;

            if inside_string {
                // Entering a string
                output.push(c);
                string_content.clear();
            } else {
                // Exiting a string
                // Remove misplaced comma inside string if any
                if string_content.ends_with(',') {
                    string_content.pop();
                    changed = true;
                    changes_made.push("Removed trailing comma inside a string".to_string());
                }
                output.push_str(&string_content);
                output.push('"');

                // Peek ahead to check for the next significant character
                let mut temp_chars = chars.clone().peekable();
                while let Some(&next_c) = temp_chars.peek() {
                    if next_c.is_whitespace() {
                        temp_chars.next();
                    } else {
                        break;
                    }
                }

                if let Some(&next_c) = temp_chars.peek() {
                    if next_c == '"' {
                        // Next string starts immediately
                        // Check if we need to insert a comma
                        let mut output_chars = output.chars().rev().skip_while(|c| c.is_whitespace());
                        if let Some(last_c) = output_chars.next() {
                            if last_c != ',' && last_c != '[' && last_c != '{' {
                                output.push(',');
                                changed = true;
                                changes_made.push("Inserted comma between adjacent strings".to_string());
                            }
                        } else {
                            // Output is empty, unlikely in this context
                            output.push(',');
                            changed = true;
                            changes_made.push("Inserted comma at the beginning".to_string());
                        }
                    }
                }
                string_content.clear();
            }
        } else {
            if inside_string {
                string_content.push(c);
            } else {
                output.push(c);
            }
        }
        prev_char = Some(c);
    }

    if changed {
        info!("Repaired JSON by making the following changes: {}", changes_made.join("; "));
    }

    Ok(output)
}

#[cfg(test)]
mod attempt_repair_comma_and_quote_accidentally_swapped_tests {
    use super::*;
    use serde_json::json;

    #[traced_test]
    fn test_comma_and_quote_accidentally_swapped() -> Result<(),JsonRepairError> {
        // value3 has the comma and the trailing quote swapped
        let input = r#"{
            "key": [
                "value1",
                "value2",
                "value3,"
                "value4",
                "value5"
            ]
        }"#;

        let expected = serde_json::json!({
            "key": [
                "value1",
                "value2",
                "value3",
                "value4",
                "value5"
            ]
        });

        let output = repair_json_comma_behavior(input)?;

        let output_json = serde_json::from_str::<serde_json::Value>(&output).unwrap();

        assert_eq!(output_json, expected);
        Ok(())
    }

    #[traced_test]
    fn test_missing_comma() -> Result<(),JsonRepairError> {
        // value5 has no comma after the quote
        let input = r#"{
            "key": [
                "value1",
                "value2",
                "value3",
                "value4",
                "value5"
                "value6",
                "value7"
            ]
        }"#;

        let expected = json!({
            "key": [
                "value1",
                "value2",
                "value3",
                "value4",
                "value5",
                "value6",
                "value7"
            ]
        });

        let output = repair_json_comma_behavior(input)?;

        let output_json 
            = serde_json::from_str::<serde_json::Value>(&output)
            .map_err(|inner| JsonRepairError::SerdeParseError { inner })?;

        assert_eq!(output_json, expected);

        Ok(())
    }
}
