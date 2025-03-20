// ---------------- [ File: src/language_model_api_url.rs ]
crate::ix!();

/// Enumeration of API URLs.
#[derive(Clone,Debug, Serialize, Deserialize)]
pub enum LanguageModelApiUrl {

    #[serde(rename = "/v1/chat/completions")]
    ChatCompletions,
}

impl fmt::Display for LanguageModelApiUrl {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageModelApiUrl::ChatCompletions => write!(f, "/v1/chat/completions"),
        }
    }
}
#[cfg(test)]
mod language_model_api_url_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn display_renders_chat_completions() {
        trace!("===== BEGIN TEST: display_renders_chat_completions =====");
        let url = LanguageModelApiUrl::ChatCompletions;
        let result = format!("{}", url);
        debug!("Formatted Display output: {}", result);
        pretty_assert_eq!(result, "/v1/chat/completions", "Display should render the correct endpoint");
        trace!("===== END TEST: display_renders_chat_completions =====");
    }

    #[traced_test]
    fn serialize_chat_completions_produces_expected_json() {
        trace!("===== BEGIN TEST: serialize_chat_completions_produces_expected_json =====");
        let url = LanguageModelApiUrl::ChatCompletions;
        let serialized = serde_json::to_string(&url)
            .expect("Failed to serialize LanguageModelApiUrl");
        debug!("Serialized JSON: {}", serialized);
        pretty_assert_eq!(serialized, r#""/v1/chat/completions""#, "Serialization should match expected JSON");
        trace!("===== END TEST: serialize_chat_completions_produces_expected_json =====");
    }

    #[traced_test]
    fn deserialize_chat_completions_from_json() {
        trace!("===== BEGIN TEST: deserialize_chat_completions_from_json =====");
        let json_str = r#""/v1/chat/completions""#;
        let deserialized: LanguageModelApiUrl = serde_json::from_str(json_str)
            .expect("Failed to deserialize LanguageModelApiUrl");
        debug!("Deserialized enum variant: {:?}", deserialized);
        match deserialized {
            LanguageModelApiUrl::ChatCompletions => {
                trace!("Correctly deserialized as ChatCompletions");
            }
        }
        trace!("===== END TEST: deserialize_chat_completions_from_json =====");
    }
}
