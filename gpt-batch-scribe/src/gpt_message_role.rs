crate::ix!();

/// Enumeration of roles in a message.
#[derive(Debug, Serialize, Deserialize)]
pub enum GptMessageRole {
    System,
    User,
}

pub(crate) mod message_role {

    use super::*;

    /// Serialize the `GptMessageRole` enum into a string.
    pub fn serialize<S>(value: &GptMessageRole, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let role_str = match value {
            GptMessageRole::System => "system",
            GptMessageRole::User => "user",
        };
        serializer.serialize_str(role_str)
    }

    /// Deserialize a string into a `GptMessageRole` enum.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<GptMessageRole, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_ref() {
            "system" => Ok(GptMessageRole::System),
            "user" => Ok(GptMessageRole::User),
            _ => Err(DeError::custom("unknown message role")),
        }
    }
}


