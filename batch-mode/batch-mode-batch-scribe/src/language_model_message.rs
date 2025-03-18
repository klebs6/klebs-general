// ---------------- [ File: src/language_model_message.rs ]
crate::ix!();

/// Individual message details in the request body.
#[derive(Getters,Setters,Clone,Debug, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct LanguageModelMessage {
    /// Role of the participant (system/user).
    #[serde(with = "message_role")]
    role: LanguageModelMessageRole,
    /// Content of the message.
    content: ChatCompletionRequestUserMessageContent,
}

impl LanguageModelMessage {

    pub fn system_message(msg: &str) -> Self {
        Self {
            role:    LanguageModelMessageRole::System,
            content: ChatCompletionRequestUserMessageContent::Text(msg.to_string()),
        }
    }

    pub fn user_message(msg: &str) -> Self {
        Self {
            role:    LanguageModelMessageRole::User,
            content: ChatCompletionRequestUserMessageContent::Text(msg.to_string()),
        }
    }

    pub fn user_message_with_image(msg: &str, image_b64: &str) -> Self {

        let image_url = ImageUrl {
            url:    image_b64.to_string(),
            detail: Some(ImageDetail::High),
        };

        let text_part  =  ChatCompletionRequestMessageContentPartText { text: msg.into() };
        let image_part = ChatCompletionRequestMessageContentPartImage { image_url };

        let parts = vec![
            ChatCompletionRequestUserMessageContentPart::Text(text_part),
            ChatCompletionRequestUserMessageContentPart::ImageUrl(image_part),
        ];

        Self {
            role:    LanguageModelMessageRole::User,
            content: ChatCompletionRequestUserMessageContent::Array(parts),
        }
    }
}

#[cfg(test)]
mod language_model_message_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn system_message_construction_produces_correct_role_and_text() {
        trace!("===== BEGIN TEST: system_message_construction_produces_correct_role_and_text =====");
        let msg_text = "System message content";
        let message = LanguageModelMessage::system_message(msg_text);
        debug!("Constructed system message: {:?}", message);

        match &message.role {
            LanguageModelMessageRole::System => {
                trace!("Message role is correct: System");
            },
            _ => {
                error!("Expected role System, got something else");
                panic!("Role mismatch");
            },
        }

        match &message.content {
            ChatCompletionRequestUserMessageContent::Text(text) => {
                debug!("Message content is a text field: {}", text);
                assert_eq!(text, msg_text, "Content text should match");
            },
            _ => {
                error!("Expected text content, found a different variant");
                panic!("Content mismatch");
            }
        }

        trace!("===== END TEST: system_message_construction_produces_correct_role_and_text =====");
    }

    #[traced_test]
    fn user_message_construction_produces_correct_role_and_text() {
        trace!("===== BEGIN TEST: user_message_construction_produces_correct_role_and_text =====");
        let msg_text = "User message content";
        let message = LanguageModelMessage::user_message(msg_text);
        debug!("Constructed user message: {:?}", message);

        match &message.role {
            LanguageModelMessageRole::User => {
                trace!("Message role is correct: User");
            },
            _ => {
                error!("Expected role User, got something else");
                panic!("Role mismatch");
            },
        }

        match &message.content {
            ChatCompletionRequestUserMessageContent::Text(text) => {
                debug!("Message content is a text field: {}", text);
                assert_eq!(text, msg_text, "Content text should match");
            },
            _ => {
                error!("Expected text content, found a different variant");
                panic!("Content mismatch");
            }
        }

        trace!("===== END TEST: user_message_construction_produces_correct_role_and_text =====");
    }

    #[traced_test]
    fn user_message_with_image_construction_includes_text_and_image() {
        trace!("===== BEGIN TEST: user_message_with_image_construction_includes_text_and_image =====");
        let msg_text = "Look at this image";
        let image_b64 = "base64image==";
        let message = LanguageModelMessage::user_message_with_image(msg_text, image_b64);
        debug!("Constructed user message with image: {:?}", message);

        match &message.role {
            LanguageModelMessageRole::User => {
                trace!("Message role is correct: User");
            },
            _ => {
                error!("Expected role User, got something else");
                panic!("Role mismatch");
            },
        }

        match &message.content {
            ChatCompletionRequestUserMessageContent::Array(parts) => {
                debug!("Message content is an array with {} part(s)", parts.len());
                assert_eq!(parts.len(), 2, "Expected two parts: text + image");
                match (&parts[0], &parts[1]) {
                    (
                        ChatCompletionRequestUserMessageContentPart::Text(ChatCompletionRequestMessageContentPartText{ text: t }),
                        ChatCompletionRequestUserMessageContentPart::ImageUrl(ChatCompletionRequestMessageContentPartImage { image_url: img }),
                    ) => {
                        debug!("Text part: {}, Image URL: {}", t, img.url);
                        assert_eq!(t, msg_text, "Text part should match original message");
                        assert_eq!(&img.url, image_b64, "Image URL should match input");
                    },
                    _ => {
                        error!("Array content did not have the expected (Text, ImageUrl) structure");
                        panic!("Parts mismatch");
                    }
                }
            },
            _ => {
                error!("Expected array content, found a different variant");
                panic!("Content mismatch");
            }
        }

        trace!("===== END TEST: user_message_with_image_construction_includes_text_and_image =====");
    }

    #[traced_test]
    fn serialization_and_deserialization_of_system_message() {
        trace!("===== BEGIN TEST: serialization_and_deserialization_of_system_message =====");
        let original = LanguageModelMessage::system_message("Hello system");
        let serialized = serde_json::to_string(&original)
            .expect("Failed to serialize system message");
        debug!("Serialized system message: {}", serialized);

        let deserialized: LanguageModelMessage = serde_json::from_str(&serialized)
            .expect("Failed to deserialize system message");
        debug!("Deserialized system message: {:?}", deserialized);

        // Compare fields
        assert_eq!(format!("{:?}", original.role), format!("{:?}", deserialized.role));
        assert_eq!(format!("{:?}", original.content), format!("{:?}", deserialized.content));

        trace!("===== END TEST: serialization_and_deserialization_of_system_message =====");
    }

    #[traced_test]
    fn serialization_and_deserialization_of_user_message_with_image() {
        trace!("===== BEGIN TEST: serialization_and_deserialization_of_user_message_with_image =====");
        let original = LanguageModelMessage::user_message_with_image("User says hi", "b64encoded==");
        let serialized = serde_json::to_string(&original)
            .expect("Failed to serialize user message with image");
        debug!("Serialized user message with image: {}", serialized);

        let deserialized: LanguageModelMessage = serde_json::from_str(&serialized)
            .expect("Failed to deserialize user message with image");
        debug!("Deserialized user message with image: {:?}", deserialized);

        // Compare fields
        assert_eq!(format!("{:?}", original.role), format!("{:?}", deserialized.role));
        assert_eq!(format!("{:?}", original.content), format!("{:?}", deserialized.content));

        trace!("===== END TEST: serialization_and_deserialization_of_user_message_with_image =====");
    }
}
