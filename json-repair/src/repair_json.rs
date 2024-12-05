crate::ix!();

pub fn repair_json_string(input: &str) -> Result<Value, JsonRepairError> {
    /*
    match repair_json_string_parallel(input) {
        Ok(repaired) => Ok(repaired),
        Err(e)       => repair_json_string_series(input),
    }
    */
    repair_json_string_series(input)
}

#[cfg(test)]
mod repair_json_tests {
    use super::*;
    use serde_json::json;

    #[traced_test]
    fn test_valid_json() {
        let input = r#"{"key": "value", "list": [1, 2, 3]}"#;
        let expected = serde_json::from_str(input).unwrap();
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_truncated_in_string() {
        let input = r#"{"key": "value"#;
        let expected = json!({"key": "value"});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_unclosed_array() {
        let input = r#"{"list": [1, 2, 3"#;
        let expected = json!({"list": [1, 2, 3]});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_unclosed_object() {
        let input = r#"{"object": {"nested": "value"#;
        let expected = json!({"object": {"nested": "value"}});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_trailing_comma_in_object() {
        let input = r#"{"key": "value",}"#;
        let expected = json!({"key": "value"});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_trailing_comma_in_array() {
        let input = r#"{"list": [1, 2, 3, ]}"#;
        let expected = json!({"list": [1, 2, 3]});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_empty_input() {
        let input = "";
        let expected = json!({});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_only_opening_brace() {
        let input = "{";
        let expected = json!({});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_only_opening_bracket() {
        let input = "[";
        let expected = json!([]);
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_unclosed_string_in_array() {
        let input = r#"["value1", "value2"#;
        let expected = json!(["value1", "value2"]);
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_truncated_in_the_middle_of_array_element() {
        let input = r#"["value1", "value2", "value"#;
        let expected = json!(["value1", "value2", "value"]);
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_nested_structures_with_truncation() {
        let input = r#"{"a": {"b": {"c": [1, 2, {"d": "e"#;
        let expected = json!({"a": {"b": {"c": [1, 2, {"d": "e"}]}}});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_truncated_number() {
        let input = r#"{"number": 1234"#;
        let expected = json!({"number": 1234});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_truncated_boolean_true() {
        let input    = r#"{"bool": tr"#;
        let output   = repair_json_string(input);
        let expected = json!({"bool": true});
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_truncated_boolean_false() {
        let input = r#"{"bool": fal"#;
        let output = repair_json_string(input);
        let expected = json!({"bool": false});
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_extra_commas_and_unclosed_structures() {
        let input = r#"{"key1": "value1", "key2": "value2", "#;
        let expected = json!({"key1": "value1", "key2": "value2"});
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_complex_truncated_json() {
        let input = r#"{
  "aesthetic_details": [
    "Focus on contrasts.",
    "Patterns in water.",
    "Intricate branches.",
  ],
  "cognitive_and_perceptual_influence": [
    "Enhances awareness.",
    "Stimulates thought.",
    "Encourages storytelling.",
  ],
  "concrete_steps_to_create_in_our_location": [
    "Identify location.",
    "Design layout.",
    "Engage historians.",
  ],
  "core_essence_and_symbolism": [
    "Represents integration of nature and civilization.",
    "Embodies the timeless flow of knowledge."
  ],
  "additional_notes": "This project aims to fu
"#;
        let expected = json!({
            "aesthetic_details": [
                "Focus on contrasts.",
                "Patterns in water.",
                "Intricate branches."
            ],
            "cognitive_and_perceptual_influence": [
                "Enhances awareness.",
                "Stimulates thought.",
                "Encourages storytelling."
            ],
            "concrete_steps_to_create_in_our_location": [
                "Identify location.",
                "Design layout.",
                "Engage historians."
            ],
            "core_essence_and_symbolism": [
                "Represents integration of nature and civilization.",
                "Embodies the timeless flow of knowledge."
            ],
            "additional_notes": "This project aims to fu"
        });
        let output = repair_json_string(input);
        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_repair_single_quote_instead_of_double_quote() {
        //value4 has a single quote instead of a double
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

        let output = repair_json_string(input);

        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_missing_comma() {

        //value5 has no comma after the quote
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

        let output = repair_json_string(input);

        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_comma_and_quote_accidentally_swapped() {
        //value3 has the comma and the trailing quote swapped
        let input = r#"{
            "key": [
                "value1",
                "value2",
                "value3,"
                "value4",
                "value5"
            ]
        }"#;

        let expected = json!({
            "key": [
                "value1",
                "value2",
                "value3",
                "value4",
                "value5"
            ]
        });

        let output = repair_json_string(input);

        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_multiple_problems() {
        //value3 has the comma and the trailing quote swapped
        //value5 has no trailing comma
        let input = r#"{
            "key": [
                "value1",
                "value2",
                "value3,"
                "value4",
                "value5"
                "value6",
                "value7",
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

        let output = repair_json_string(input);

        assert_expected_matches_output_result(input,&output,&expected);
    }

    #[traced_test]
    fn test_repair_single_quote_in_keys_and_values() {
        let input = r#"{
            'key1': 'value1',
            'key2': "value2",
            "key3": 'value3',
            "text": "Don't stop believing",
            'another_text': 'It\'s a kind of magic',
            "nested": {
                'inner_key': 'inner_value'
            }
        }"#;

        let expected = json!({
            "key1": "value1",
            "key2": "value2",
            "key3": "value3",
            "text": "Don't stop believing",
            "another_text": "It's a kind of magic",
            "nested": {
                "inner_key": "inner_value"
            }
        });

        let output = repair_json_string(input);

        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_repair_mixed_quotes_and_escaped_quotes() {
        let input = r#"{
            "message": 'He said, "It\'s a sunny day!"',
            'reply': "Yes, it\'s beautiful."
        }"#;

        let expected = json!({
            "message": "He said, \"It's a sunny day!\"",
            "reply": "Yes, it's beautiful."
        });

        let output = repair_json_string(input);

        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_brace_instead_of_bracket() {
        let input = r#"{
          "tag1": [
            "item1",
            "item2",
            "item3",
            "tag4"
          },
          "tag2": [
            "item1",
            "item2",
            "item3",
            "item4"
          ]
        }"#;

        let expected = json!({
            "tag1": [
                "item1",
                "item2",
                "item3",
                "tag4"
            ],
            "tag2": [
                "item1",
                "item2",
                "item3",
                "item4"
            ]
        });

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_control_character_error() {
        let input = "{ \"text\": \"This is a test\u{0001}string with control characters\" }";

        let expected = json!({
            "text": "This is a teststring with control characters"
        });

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_comma_inside_list() {
        let input = r#"{
          "tag": [
            "item1",
            "item2",
            "item3"
            "item4",
            "item5",
            "item6",
            "item7",
            "item8",
            "item9",
            "item10",
            "item11",
            "item12",
            "item13",
            "item14",
            "item15"
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
                "item10",
                "item11",
                "item12",
                "item13",
                "item14",
                "item15"
            ]
        });

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_unexpected_eof_inside_list() {
        let input = r#"{
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
                "item10",
                "item11",
                "item12",
                "iteEOF
        "#;

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
                "item10",
                "item11",
                "item12",
                "iteEOF"
            ]
        });

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_duplicate_quote_to_close_list_item() {
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

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[test]
    fn test_missing_closing_double_quote_but_comma_present() -> Result<(), JsonRepairError> {
        let input = r#"{
          "tag": [
            "item1",
            "item2",
            "item3",
            "item4",
            "item5,
            "item6",
            "item7",
            "item8",
            "item9",
            "item10",
            "item11",
            "item12"
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
                "item10",
                "item11",
                "item12"
            ]
        });

        let output = repair_json_string_series(input)?;

        // Parse output as JSON Value
        let output_json: Value = output;

        assert_eq!(output_json, expected);

        Ok(())
    }

    #[traced_test]
    fn test_eof_in_between_lists() {
        let input = r#"{
          "tag": [
            "item1",
            "item2",
            // ... more items ...
            "item20"
          ],
          "a"#;

        let expected = json!({
            "tag": [
                "item1",
                "item2",
                // ... more items ...
                "item20"
            ],
            "a": null
        });

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_bad_quote_character() {
        let input = r#"{
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
                "item10",
                "item11',
                "item12",
                "item13",
                "item14",
                "item15",
                "item16",
                "item17",
                "item18",
                "item19",
                "item20"
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
                "item10",
                "item11",
                "item12",
                "item13",
                "item14",
                "item15",
                "item16",
                "item17",
                "item18",
                "item19",
                "item20"
            ]
        });

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_eof_found_midway_through_array_tag() {
        let input = r#"{
          "tag1": [
            "item1",
            // ... more items ...
            "item20"
          ],
          "tag2"#;

        let expected = json!({
            "tag1": [
                "item1",
                // ... more items ...
                "item20"
            ],
            "tag2": null
        });

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_eof_found_midway_through_array_item() {
        let input = r#"{
          "tag": [
            "item1",
            // ... more items ...
            "itEOF
        "#;

        let expected = json!({
            "tag": [
                "item1",
                // ... more items ...
                "itEOF"
            ]
        });

        let output = repair_json_string(input);
        assert_expected_matches_output_result(input, &output, &expected);
    }
}
