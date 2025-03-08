// ---------------- [ File: src/batch_message_content.rs ]
crate::ix!();

#[derive(Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(transparent)]
pub struct BatchMessageContent {
    content:            String,
    #[serde(skip)]
    sanitized_json_str: OnceCell<String>,
}

unsafe impl Send for BatchMessageContent {}
unsafe impl Sync for BatchMessageContent {}

impl PartialEq<str> for BatchMessageContent {

    fn eq(&self, other: &str) -> bool{
        self.as_ref().eq(other)
    }
}

impl AsRef<str> for BatchMessageContent {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

impl BatchMessageContent {

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
    fn parse_inner_json(&self, strategy: JsonParsingStrategy) -> Result<Value, JsonParseError> {
        let sanitized_json_str = self.get_sanitized_json_str();
        match serde_json::from_str(&sanitized_json_str) {
            Ok(json_value) => Ok(json_value),
            Err(e) => {
                warn!(
                    "Failed to parse JSON string. Will try to repair it. Error: {}",
                     e
                );

                match strategy {
                    JsonParsingStrategy::WithRepair => {
                        // Attempt to repair the JSON
                        match repair_json_with_known_capitalized_sentence_fragment_list_items(&sanitized_json_str) {
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
    pub fn extract_clean_parse_json(&self) -> Result<Value, JsonParseError> {
        self.parse_inner_json(JsonParsingStrategy::WithoutRepair)
    }

    /// Extracts and parses JSON, attempting to repair on failure.
    pub fn extract_clean_parse_json_with_repair(&self) -> Result<Value, JsonParseError> {
        self.parse_inner_json(JsonParsingStrategy::WithRepair)
    }
}
