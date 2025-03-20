// ---------------- [ File: src/language_model_request_body.rs ]
crate::ix!();

/// Body details of the API request.
#[derive(Getters,Setters,Clone,Debug, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct LanguageModelRequestBody {

    /// Model used for the request.
    #[serde(with = "model_type")]
    model: LanguageModelType,

    /// Array of messages exchanged in the request.
    messages: Vec<LanguageModelMessage>,

    /// Maximum number of tokens to be used by the model.
    max_completion_tokens: u32,
}

impl LanguageModelRequestBody {

    pub fn mock() -> Self {
        LanguageModelRequestBody {
            model:                 LanguageModelType::Gpt4o,
            messages:              vec![],
            max_completion_tokens: 128,
        }
    }

    pub fn default_max_tokens() -> u32 {
        //1024 
        8192
    }

    pub fn default_max_tokens_given_image(_image_b64: &str) -> u32 {
        //TODO: is this the right value?
        2048
    }

    pub fn new_basic(model: LanguageModelType, system_message: &str, user_message: &str) -> Self {
        Self {
            model,
            messages: vec![
                LanguageModelMessage::system_message(system_message),
                LanguageModelMessage::user_message(user_message),
            ],
            max_completion_tokens: Self::default_max_tokens(),
        }
    }

    pub fn new_with_image(model: LanguageModelType, system_message: &str, user_message: &str, image_b64: &str) -> Self {
        Self {
            model,
            messages: vec![
                LanguageModelMessage::system_message(system_message),
                LanguageModelMessage::user_message_with_image(user_message,image_b64),
            ],
            max_completion_tokens: Self::default_max_tokens_given_image(image_b64),
        }
    }
}

#[cfg(test)]
mod language_model_request_body_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn mock_produces_gpt4o_empty_messages_128_tokens() {
        trace!("===== BEGIN TEST: mock_produces_gpt4o_empty_messages_128_tokens =====");
        let body = LanguageModelRequestBody::mock();
        debug!("Mock body: {:?}", body);

        match body.model {
            LanguageModelType::Gpt4o => trace!("Correct model: Gpt4o"),
            _ => panic!("Expected LanguageModelType::Gpt4o"),
        }

        assert!(
            body.messages.is_empty(),
            "Mock body should have no messages"
        );
        pretty_assert_eq!(
            body.max_completion_tokens, 128,
            "Mock body should have max_completion_tokens=128"
        );

        trace!("===== END TEST: mock_produces_gpt4o_empty_messages_128_tokens =====");
    }

    #[traced_test]
    fn default_max_tokens_returns_8192() {
        trace!("===== BEGIN TEST: default_max_tokens_returns_8192 =====");
        let tokens = LanguageModelRequestBody::default_max_tokens();
        debug!("default_max_tokens: {}", tokens);
        pretty_assert_eq!(tokens, 8192, "default_max_tokens should return 8192");
        trace!("===== END TEST: default_max_tokens_returns_8192 =====");
    }

    #[traced_test]
    fn default_max_tokens_given_image_returns_2048() {
        trace!("===== BEGIN TEST: default_max_tokens_given_image_returns_2048 =====");
        let image_b64 = "fake_base64_image_data";
        let tokens = LanguageModelRequestBody::default_max_tokens_given_image(image_b64);
        debug!("default_max_tokens_given_image: {}", tokens);
        pretty_assert_eq!(
            tokens, 2048,
            "default_max_tokens_given_image should return 2048"
        );
        trace!("===== END TEST: default_max_tokens_given_image_returns_2048 =====");
    }

    #[traced_test]
    fn new_basic_sets_provided_model_and_messages_and_uses_default_tokens() {
        trace!("===== BEGIN TEST: new_basic_sets_provided_model_and_messages_and_uses_default_tokens =====");
        let model = LanguageModelType::Gpt4o;
        let system_message = "System says hello";
        let user_message = "User says hi";
        let body = LanguageModelRequestBody::new_basic(model.clone(), system_message, user_message);
        debug!("Constructed body: {:?}", body);

        match body.model {
            LanguageModelType::Gpt4o => trace!("Model is Gpt4o as expected"),
            _ => panic!("Expected LanguageModelType::Gpt4o"),
        }
        pretty_assert_eq!(body.messages.len(), 2, "Should have 2 messages total");
        match &body.messages[0].content() {
            ChatCompletionRequestUserMessageContent::Text(text) => {
                pretty_assert_eq!(text, system_message, "System message mismatch");
            },
            _ => panic!("Expected text content for system message"),
        }
        match &body.messages[1].content() {
            ChatCompletionRequestUserMessageContent::Text(text) => {
                pretty_assert_eq!(text, user_message, "User message mismatch");
            },
            _ => panic!("Expected text content for user message"),
        }

        pretty_assert_eq!(
            *body.max_completion_tokens(),
            LanguageModelRequestBody::default_max_tokens(),
            "max_completion_tokens should match default"
        );

        trace!("===== END TEST: new_basic_sets_provided_model_and_messages_and_uses_default_tokens =====");
    }

    #[traced_test]
    fn new_with_image_sets_provided_model_and_messages_and_uses_image_default_tokens() {
        trace!("===== BEGIN TEST: new_with_image_sets_provided_model_and_messages_and_uses_image_default_tokens =====");
        let model = LanguageModelType::Gpt4o;
        let system_message = "System with image instructions";
        let user_message = "User requests image";
        let image_b64 = "fake_image_b64";
        let body = LanguageModelRequestBody::new_with_image(model.clone(), system_message, user_message, image_b64);
        debug!("Constructed body with image: {:?}", body);

        match body.model {
            LanguageModelType::Gpt4o => trace!("Model is Gpt4o as expected"),
            _ => panic!("Expected LanguageModelType::Gpt4o"),
        }
        pretty_assert_eq!(body.messages.len(), 2, "Should have 2 messages total");
        match &body.messages[0].content() {
            ChatCompletionRequestUserMessageContent::Text(text) => {
                pretty_assert_eq!(text, system_message, "System message mismatch");
            },
            _ => panic!("Expected text content for system message"),
        }

        match &body.messages[1].content() {
            ChatCompletionRequestUserMessageContent::Array(parts) => {
                pretty_assert_eq!(parts.len(), 2, "Expected text + image parts");
            },
            _ => panic!("Expected array content for user message with image"),
        }

        pretty_assert_eq!(
            body.max_completion_tokens,
            LanguageModelRequestBody::default_max_tokens_given_image(image_b64),
            "max_completion_tokens should match default for images"
        );

        trace!("===== END TEST: new_with_image_sets_provided_model_and_messages_and_uses_image_default_tokens =====");
    }

    #[traced_test]
    fn serialization_and_deserialization_round_trip() {
        trace!("===== BEGIN TEST: serialization_and_deserialization_round_trip =====");
        let original = LanguageModelRequestBody::new_basic(
            LanguageModelType::Gpt4o,
            "System Info",
            "User Query"
        );
        let serialized = serde_json::to_string(&original)
            .expect("Failed to serialize LanguageModelRequestBody");
        debug!("Serialized: {}", serialized);

        let deserialized: LanguageModelRequestBody = serde_json::from_str(&serialized)
            .expect("Failed to deserialize LanguageModelRequestBody");
        debug!("Deserialized: {:?}", deserialized);

        // Compare essential fields
        pretty_assert_eq!(format!("{:?}", original.model), format!("{:?}", deserialized.model));
        pretty_assert_eq!(
            original.messages.len(),
            deserialized.messages.len(),
            "Messages length mismatch after round-trip"
        );
        pretty_assert_eq!(
            original.max_completion_tokens,
            deserialized.max_completion_tokens,
            "max_completion_tokens mismatch after round-trip"
        );

        trace!("===== END TEST: serialization_and_deserialization_round_trip =====");
    }
}
