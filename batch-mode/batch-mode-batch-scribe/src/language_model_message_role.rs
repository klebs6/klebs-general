// ---------------- [ File: src/language_model_message_role.rs ]
crate::ix!();

/// Enumeration of roles in a message.
#[derive(Clone,Debug, Serialize, Deserialize)]
pub enum LanguageModelMessageRole {
    System,
    User,
}

pub(crate) mod message_role {

    use super::*;

    /// Serialize the `LanguageModelMessageRole` enum into a string.
    pub fn serialize<S>(value: &LanguageModelMessageRole, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let role_str = match value {
            LanguageModelMessageRole::System => "system",
            LanguageModelMessageRole::User => "user",
        };
        serializer.serialize_str(role_str)
    }

    /// Deserialize a string into a `LanguageModelMessageRole` enum.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<LanguageModelMessageRole, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_ref() {
            "system" => Ok(LanguageModelMessageRole::System),
            "user" => Ok(LanguageModelMessageRole::User),
            _ => Err(DeError::custom("unknown message role")),
        }
    }
}
