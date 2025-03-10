// ---------------- [ File: src/message_role.rs ]
crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageRole {
    Assistant,
    User,
    System,
    Unknown(String),
}

impl<'de> Deserialize<'de> for MessageRole {
    fn deserialize<D>(deserializer: D) -> Result<MessageRole, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "assistant" => Ok(MessageRole::Assistant),
            "user" => Ok(MessageRole::User),
            "system" => Ok(MessageRole::System),
            other => Ok(MessageRole::Unknown(other.to_string())),
        }
    }
}

impl Serialize for MessageRole {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            MessageRole::Assistant => "assistant",
            MessageRole::User => "user",
            MessageRole::System => "system",
            MessageRole::Unknown(other) => other.as_str(),
        };
        serializer.serialize_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test suite for MessageRole
    #[test]
    fn test_message_role_deserialization() {
        // Known roles
        let roles = vec!["assistant", "user", "system"];
        let expected_roles = vec![
            MessageRole::Assistant,
            MessageRole::User,
            MessageRole::System,
        ];

        for (role_str, expected_role) in roles.iter().zip(expected_roles.iter()) {
            let json = format!("\"{}\"", role_str);
            let role: MessageRole = serde_json::from_str(&json).unwrap();
            assert_eq!(&role, expected_role);
        }

        // Unknown role
        let json = "\"unknown_role\"";
        let role: MessageRole = serde_json::from_str(json).unwrap();
        assert_eq!(role, MessageRole::Unknown("unknown_role".to_string()));

        // Empty string as role
        let json = "\"\"";
        let role: MessageRole = serde_json::from_str(json).unwrap();
        assert_eq!(role, MessageRole::Unknown("".to_string()));

        // Invalid role (non-string)
        let json = "123";
        let result: Result<MessageRole, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Null role
        let json = "null";
        let result: Result<MessageRole, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
