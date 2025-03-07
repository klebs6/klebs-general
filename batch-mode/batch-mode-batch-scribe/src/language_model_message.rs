crate::ix!();

/// Individual message details in the request body.
#[derive(Debug, Serialize, Deserialize)]
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

        Self {
            role:    LanguageModelMessageRole::User,
            content: ChatCompletionRequestUserMessageContent::Array(vec![
                ChatCompletionRequestMessageContentPart::Text(msg.into()),
                ChatCompletionRequestMessageContentPart::ImageUrl(ChatCompletionRequestMessageContentPartImage {
                    image_url: ImageUrl {
                        url:    image_b64.to_string(),
                        detail: Some(ImageDetail::High),
                    }
                }),
            ]),
        }
    }
}
