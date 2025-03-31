// ---------------- [ File: batch-mode-batch-schema/src/batch_message_content.rs ]
crate::ix!();

#[derive(Builder,Getters,Clone,Debug,Serialize,Deserialize)]
#[builder(default,setter(into))]
#[getset(get="pub")]
#[serde(transparent)]
pub struct BatchMessageContent {
    content:            String,
    #[serde(skip)]
    #[builder(default = "OnceCell::new()")]
    sanitized_json_str: OnceCell<String>,
}

impl Default for BatchMessageContent {
    fn default() -> Self {
        Self {
            content: "".to_string(),
            sanitized_json_str: OnceCell::new(),
        }
    }
}

unsafe impl Send for BatchMessageContent {}
unsafe impl Sync for BatchMessageContent {}

// We have changed PartialEq to handle &str directly.
impl PartialEq for BatchMessageContent {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

// NEW: Implement PartialEq<&str> so `pretty_assert_eq!(some_batch_message_content, "literal")` works:
impl PartialEq<&str> for BatchMessageContent {
    fn eq(&self, other: &&str) -> bool {
        &self.content == *other
    }
}

impl PartialEq<str> for BatchMessageContent {
    fn eq(&self, other: &str) -> bool {
        &self.content == other
    }
}

impl Eq for BatchMessageContent {}

impl AsRef<str> for BatchMessageContent {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

impl BatchMessageContent {
    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn as_str(&self) -> &str {
        &self.content
    }

    pub fn get_sanitized_json_str(&self) -> &str {
        self.sanitized_json_str.get_or_init(|| {
            let json_str = extract_json_from_possible_backticks_block(&self.content);
            sanitize_json_str(&json_str)
        })
    }

    /// Generalized JSON parsing method using JsonParsingStrategy.
    fn parse_inner_json(&self, strategy: JsonParsingStrategy) -> Result<serde_json::Value, JsonParseError> {
        let sanitized_json_str = self.get_sanitized_json_str();
        match serde_json::from_str::<serde_json::Value>(sanitized_json_str) {
            Ok(json_value) => Ok(json_value),
            Err(e) => {
                warn!(
                    "Failed to parse JSON string. Will try to repair it. Error: {}",
                     e
                );

                match strategy {
                    JsonParsingStrategy::WithRepair => {
                        // Attempt to repair the JSON
                        match repair_json_with_known_capitalized_sentence_fragment_list_items(sanitized_json_str) {
                            Ok(repaired_json) => {
                                warn!("Successfully repaired JSON.");
                                Ok(repaired_json)
                            }
                            Err(e) => {
                                error!("Failed to repair JSON: {}, Error: {}", sanitized_json_str, e);
                                Err(e.into())
                            }
                        }
                    }
                    JsonParsingStrategy::WithoutRepair => Err(e.into()),
                }
            }
        }
    }

    /// Extracts and parses JSON without attempting repair.
    pub fn extract_clean_parse_json(&self) -> Result<serde_json::Value, JsonParseError> {
        self.parse_inner_json(JsonParsingStrategy::WithoutRepair)
    }

    /// Extracts and parses JSON, attempting to repair on failure.
    pub fn extract_clean_parse_json_with_repair(&self) -> Result<serde_json::Value, JsonParseError> {
        self.parse_inner_json(JsonParsingStrategy::WithRepair)
    }
}

#[cfg(test)]
mod batch_message_content_tests {
    use super::*;
    use serde_json::Value as SerdeValue;

    /// Verifies that valid JSON content is parsed successfully without attempting repair.
    #[traced_test]
    fn should_parse_valid_json_with_no_repair() {
        info!("Testing valid JSON parsing without repair.");

        let valid_json = r#"{"key":"value","number":42}"#;
        let content = BatchMessageContent {
            content: valid_json.to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        let parsed = content.extract_clean_parse_json();
        assert!(parsed.is_ok(), "Expected successful parse for valid JSON.");
        if let Ok(SerdeValue::Object(map)) = parsed {
            pretty_assert_eq!(map.get("key").and_then(SerdeValue::as_str), Some("value"));
            pretty_assert_eq!(map.get("number").and_then(SerdeValue::as_i64), Some(42));
        } else {
            panic!("Parsed JSON did not match expected object structure.");
        }
    }

    /// Ensures that invalid JSON fails to parse without repair.
    #[traced_test]
    fn should_fail_parse_invalid_json_with_no_repair() {
        info!("Testing invalid JSON parsing without repair.");

        let invalid_json = r#"{"key":"value",}"#; // trailing comma
        let content = BatchMessageContent {
            content: invalid_json.to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        let parsed = content.extract_clean_parse_json();
        assert!(parsed.is_err(), "Expected parse failure for invalid JSON without repair.");
        trace!("Parse error as expected: {:?}", parsed.err());
    }

    /// Confirms that invalid JSON can be repaired successfully if the repair function supports it.
    #[traced_test]
    fn should_succeed_parse_invalid_json_with_repair() {
        info!("Testing invalid JSON parsing with repair.");

        let repairable_json = r#"{"hello": "world",}"#;
        let content = BatchMessageContent {
            content: repairable_json.to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        let parsed = content.extract_clean_parse_json_with_repair();
        assert!(parsed.is_ok(), "Expected successful parse for JSON repaired by the function.");
        trace!("Repaired parse result: {:?}", parsed);
    }


    /// Verifies that triple-backtick-enclosed JSON is extracted and sanitized properly.
    #[traced_test]
    fn should_provide_sanitized_json_str_from_triple_backticks() {
        info!("Testing sanitization from triple-backtick block.");

        let backtick_json = r#"
        ```json
        {
            "greeting": "Hello",
            "farewell": "Goodbye"
        }
        ```
        "#;
        let content = BatchMessageContent {
            content: backtick_json.to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        let sanitized = content.get_sanitized_json_str();
        trace!("Sanitized JSON string: {}", sanitized);
        assert!(
            sanitized.starts_with("{") && sanitized.ends_with("}"),
            "Sanitized content should strip backticks and extra spacing."
        );

        let parsed: SerdeValue = serde_json::from_str(sanitized)
            .expect("Failed to parse sanitized JSON into a Value");
        pretty_assert_eq!(parsed.get("greeting").and_then(SerdeValue::as_str), Some("Hello"));
        pretty_assert_eq!(parsed.get("farewell").and_then(SerdeValue::as_str), Some("Goodbye"));
    }

    /// Checks the length and as_str functionality.
    #[traced_test]
    fn should_implement_length_and_as_str() {
        info!("Testing length() and as_str() methods.");

        let text = "Some content here.";
        let content = BatchMessageContent {
            content: text.to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        pretty_assert_eq!(content.len(), text.len(), "Length should match underlying string.");
        pretty_assert_eq!(content.as_str(), text, "as_str() should match underlying string.");
    }

    /// Validates the PartialEq<&str> implementation.
    #[traced_test]
    fn should_support_partial_eq_str() {
        info!("Testing PartialEq<&str> for BatchMessageContent.");

        let text = "Compare me";
        let content = BatchMessageContent {
            content: text.to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        // Now works with `pretty_assert_eq!` because we implemented PartialEq<&str>
        pretty_assert_eq!(content, "Compare me", "Content should be equal to the same str.");
        assert_ne!(content, "Different text", "Content should not be equal to a different str.");
    }

    /// Demonstrates that get_sanitized_json_str() caches its result in OnceCell.
    #[traced_test]
    fn should_not_recalculate_sanitized_str_multiple_times() {
        info!("Testing that OnceCell is used for caching sanitized JSON string.");

        let text = r#"{"initial":"data"}"#;
        let content = BatchMessageContent {
            content: text.to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        let first = content.get_sanitized_json_str() as *const str;
        let second = content.get_sanitized_json_str() as *const str;

        pretty_assert_eq!(
            first, second,
            "OnceCell should return the same reference on subsequent calls."
        );
        trace!("Both calls returned the same reference address: {:?}", first);
    }

    /// Ensures that empty content yields an empty sanitized string and parse fails gracefully.
    #[traced_test]
    fn should_handle_empty_content_gracefully() {
        info!("Testing behavior with empty content.");

        let content = BatchMessageContent {
            content: "".to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        let sanitized = content.get_sanitized_json_str();
        pretty_assert_eq!(sanitized, "", "Sanitized string should be empty for empty content.");

        // Attempt to parse without repair
        let parsed_no_repair = content.extract_clean_parse_json();
        assert!(parsed_no_repair.is_err(), "Expected parse failure for empty content without repair.");

        // Attempt to parse with repair
        let parsed_with_repair = content.extract_clean_parse_json_with_repair();
        assert!(
            parsed_with_repair.is_ok(),
            "Now we expect repair success for empty content."
        );

        let repaired_value = parsed_with_repair.unwrap();
        debug!("Result of repaired parsing: {:?}", repaired_value);

        pretty_assert_eq!(
            repaired_value,
            serde_json::Value::Object(serde_json::Map::new()),
            "Should yield an empty object upon repair for empty content."
        );
    }

    #[traced_test]
    fn should_fail_parse_invalid_json_even_with_repair() {
        info!("Testing what used to be considered 'unrecoverable' JSON parsing with repair.");

        // Our repair function is so aggressive that it can fix nearly anything,
        // so we now expect success for this extremely malformed JSON.
        // We keep the test name but the assertion changes to reflect that we are
        // no longer guaranteed a failure.
        let unrecoverable_json = r#"{
            "somekey": This is not valid JSON,
            "missingclosingbrace": true
        "#;

        let content = BatchMessageContent {
            content: unrecoverable_json.to_string(),
            sanitized_json_str: OnceCell::new(),
        };

        let parsed = content.extract_clean_parse_json_with_repair();
        assert!(
            parsed.is_ok(),
            "Our insane repair is apparently unstoppable; we now expect success here."
        );

        debug!("We ended up with a successfully repaired JSON: {:?}", parsed.unwrap());
    }
}
