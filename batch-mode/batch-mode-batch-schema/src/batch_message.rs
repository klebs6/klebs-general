// ---------------- [ File: batch-mode-batch-schema/src/batch_message.rs ]
crate::ix!();

#[derive(Default,Builder,Getters,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct BatchMessage {
    role:    MessageRole,
    content: BatchMessageContent,

    #[builder(default)]
    refusal: Option<String>,
}

/**
  If you need to build `BatchMessage` from a `ChatCompletionResponseMessage`,
  add an explicit `From<ChatCompletionResponseMessage>` conversion.

  This fixes the error:
    "the trait bound `BatchMessage: From<ChatCompletionResponseMessage>` is not satisfied"

  so that e.g. `BatchChoiceBuilder::default().message(invalid_msg).build()?`
  works (because `invalid_msg` is a `ChatCompletionResponseMessage`, and
  we want to convert it into `BatchMessage`).
*/
impl From<ChatCompletionResponseMessage> for BatchMessage {
    fn from(msg: ChatCompletionResponseMessage) -> Self {
        // Map the Role::User / Role::Assistant / ... -> MessageRole
        let mapped_role = match msg.role {
            Role::System => MessageRole::System,
            Role::Assistant => MessageRole::Assistant,
            Role::User => MessageRole::User,
            Role::Tool => MessageRole::Tool,
            Role::Function => MessageRole::Function,
        };
        // Build the content
        // (Take `msg.content.unwrap_or_default()` if content is an Option<String> in ChatCompletionResponseMessage)
        let built_content = BatchMessageContentBuilder::default()
            .content(msg.content.unwrap_or_default())
            .build()
            .unwrap();
        // Now produce a `BatchMessage`
        BatchMessageBuilder::default()
            .role(mapped_role)
            .content(built_content)
            .refusal(None)  // If you have no direct "refusal" logic from msg
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[traced_test]
    fn test_batch_message_deserialization() {
        info!("Starting test: test_batch_message_deserialization");

        // Message with all fields
        let json = r#"{
            "role": "assistant",
            "content": "Hello, world!",
            "refusal": null
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(message.role(), &MessageRole::Assistant);
        pretty_assert_eq!(message.content(), "Hello, world!");
        pretty_assert_eq!(*message.refusal(), None);

        // Message with refusal
        let json = r#"{
            "role": "assistant",
            "content": "I'm sorry, but I cannot assist with that request.",
            "refusal": "Policy refusal"
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        // FIX: do not deref an Option<&String> by '*message.refusal()'
        pretty_assert_eq!(*message.refusal(), Some("Policy refusal".to_string()));

        // Message with unknown role
        let json = r#"{
            "role": "unknown_role",
            "content": "Content with unknown role",
            "refusal": null
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(
            message.role(),
            &MessageRole::Unknown("unknown_role".to_string())
        );

        // Message with missing refusal field
        let json = r#"{
            "role": "assistant",
            "content": "Content without refusal"
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(*message.refusal(), None);

        // Message with empty content
        let json = r#"{
            "role": "assistant",
            "content": "",
            "refusal": null
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        pretty_assert_eq!(message.content(), "");

        // Message with invalid role (non-string)
        let json = r#"{
            "role": 123,
            "content": "Invalid role",
            "refusal": null
        }"#;
        let result: Result<BatchMessage, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Message with missing content field
        let json = r#"{
            "role": "assistant",
            "refusal": null
        }"#;
        let result: Result<BatchMessage, _> = serde_json::from_str(json);
        assert!(result.is_err());

        info!("Finished test: test_batch_message_deserialization");
    }
}
