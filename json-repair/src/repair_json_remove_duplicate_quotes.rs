crate::ix!();

pub fn repair_json_remove_duplicate_quotes(input: &str) -> Result<String, JsonRepairError> {
    let mut repaired    = String::with_capacity(input.len());
    let mut chars       = input.chars().peekable();
    let mut in_string   = false;
    let mut escape_next = false;

    let mut changed = false;

    while let Some(c) = chars.next() {
        if c == '\\' && !escape_next {
            // Start of an escape sequence
            escape_next = true;
            repaired.push(c);
            continue;
        }

        if c == '"' && !escape_next {
            if in_string {
                // Skip any duplicate quotes
                let mut skip_quotes = false;
                while let Some(&'"') = chars.peek() {
                    chars.next();
                    skip_quotes = true;
                }

                if skip_quotes {
                    changed = true;
                }

                // Peek ahead to check the next non-space character
                let mut peek_chars = chars.clone();
                let mut next_non_space = None;
                while let Some(&next_c) = peek_chars.peek() {
                    if next_c.is_whitespace() {
                        peek_chars.next();
                    } else {
                        next_non_space = Some(next_c);
                        break;
                    }
                }

                if let Some(next_c) = next_non_space {
                    if [',', '}', ']', ':'].contains(&next_c) {
                        // Valid end of string
                        in_string = false;
                        repaired.push(c);
                    } else {
                        // Unescaped quote inside string, remove it
                        changed = true;
                        continue;
                    }
                } else {
                    // End of input, assume end of string
                    in_string = false;
                    repaired.push(c);
                }
            } else {
                // Start of string
                in_string = true;
                repaired.push(c);
            }
            escape_next = false;
        } else {
            if escape_next {
                escape_next = false;
            }
            repaired.push(c);
        }
    }

    if changed {
        info!("Removed duplicate or invalid quotes in JSON.");
    }

    Ok(repaired)
}

#[cfg(test)]
mod repair_json_remove_duplicate_quotes_tests {
    use super::*;

    #[traced_test]
    fn test_remove_duplicate_quotes_on_duplicate_quote_to_close_list_item() -> Result<(),JsonRepairError> {
        let input = r#"{
          "tag": [
            "item1",
            "item2",
            "item3",
            "item4",
            "item5"",
            "item6",
            "item7",
            "item8",
            "item9",
            "item10"
          ]
        }"#;

        let expected = json!({
            "tag": [
                "item1",
                "item2",
                "item3",
                "item4",
                "item5",
                "item6",
                "item7",
                "item8",
                "item9",
                "item10"
            ]
        });

        let output = json5::from_str(&repair_json_remove_duplicate_quotes(input)?)
            .map_err(|_| JsonRepairError::CouldNotConvertTheOutputOfDuplicateQuoteRemovalToJson);

        assert_expected_matches_output_result(input, &output, &expected);

        Ok(())
    }

    #[traced_test]
    fn test_no_duplicate_quotes() -> Result<(),JsonRepairError> {
        let input = "{\"key\": \"value\"}";
        let expected = "{\"key\": \"value\"}";
        let output = repair_json_remove_duplicate_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_duplicate_quotes_at_end_of_string() -> Result<(),JsonRepairError> {
        let input = "{\"key\": \"value\"\"}";
        let expected = "{\"key\": \"value\"}";
        let output = repair_json_remove_duplicate_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_duplicate_quotes_inside_string() -> Result<(),JsonRepairError> {
        let input = "{\"key\": \"val\"\"ue\"}";
        let expected = "{\"key\": \"value\"}";
        let output = repair_json_remove_duplicate_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_escaped_quotes() -> Result<(),JsonRepairError> {
        let input = "{\"key\": \"value\\\"\"\"}";
        let expected = "{\"key\": \"value\\\"\"}";
        let output = repair_json_remove_duplicate_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_quotes_inside_string() -> Result<(),JsonRepairError> {
        let input = "{\"key\": \"He said, \\\"Hello\\\"\"}";
        let expected = "{\"key\": \"He said, \\\"Hello\\\"\"}";
        let output = repair_json_remove_duplicate_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_empty_string_with_duplicate_quotes() -> Result<(),JsonRepairError> {
        let input = "{\"key\": \"\"\"}";
        let expected = "{\"key\": \"\"}";
        let output = repair_json_remove_duplicate_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_string_with_only_quotes() -> Result<(),JsonRepairError> {
        warn!("we may want to check this test's expeted output. what should the expected behavior be?");
        let input = "{\"key\": \"\"\"\"\"}";
        let expected = "{\"key\": \"\"}";
        let output = repair_json_remove_duplicate_quotes(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_multiple_consecutive_duplicate_quotes() -> Result<(),JsonRepairError> {
        let input = "{\"key\": \"value\"\"\"\"}";
        let expected = "{\"key\": \"value\"}";
        let output = repair_json_remove_duplicate_quotes(&input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[traced_test]
    fn test_duplicate_quotes_in_nested_strings() -> Result<(),JsonRepairError> {
        let input = "{\"key\": {\"nested_key\": \"nested_value\"\"}}";
        let expected = "{\"key\": {\"nested_key\": \"nested_value\"}}";
        let output = repair_json_remove_duplicate_quotes(&input)?;
        assert_eq!(output, expected);
        Ok(())
    }
}
