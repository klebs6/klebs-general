// ---------------- [ File: src/batch_message.rs ]
crate::ix!();

#[derive(Debug,Serialize,Deserialize)]
pub struct BatchMessage {
    role:    MessageRole,
    content: BatchMessageContent,
    refusal: Option<String>,
}

impl BatchMessage {

    pub fn role(&self) -> &MessageRole {
        &self.role
    }

    pub fn content(&self) -> &BatchMessageContent {
        &self.content
    }

    pub fn refusal(&self) -> Option<&String> {
        self.refusal.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Test suite for BatchMessage
    #[test]
    fn test_batch_message_deserialization() {
        // Message with all fields
        let json = r#"{
            "role": "assistant",
            "content": "Hello, world!",
            "refusal": null
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        assert_eq!(message.role(), &MessageRole::Assistant);
        assert_eq!(message.content(), "Hello, world!");
        assert_eq!(message.refusal(), None);

        // Message with refusal
        let json = r#"{
            "role": "assistant",
            "content": "I'm sorry, but I cannot assist with that request.",
            "refusal": "Policy refusal"
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        assert_eq!(message.refusal(), Some(&"Policy refusal".to_string()));

        // Message with unknown role
        let json = r#"{
            "role": "unknown_role",
            "content": "Content with unknown role",
            "refusal": null
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        assert_eq!(
            message.role(),
            &MessageRole::Unknown("unknown_role".to_string())
        );

        // Message with missing refusal field
        let json = r#"{
            "role": "assistant",
            "content": "Content without refusal"
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        assert_eq!(message.refusal(), None);

        // Message with empty content
        let json = r#"{
            "role": "assistant",
            "content": "",
            "refusal": null
        }"#;
        let message: BatchMessage = serde_json::from_str(json).unwrap();
        assert_eq!(message.content(), "");

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
    }
}
